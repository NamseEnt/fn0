#include <atomic>
#include <algorithm>
#include <chrono>
#include <cstdlib>
#include <cstring>
#include <exception>
#include <memory>
#include <string>
#include <string_view>
#include <signal.h>
#include <unistd.h>

#include "rocksdb/db.h"
#include "rocksdb/env.h"
#include "rocksdb/cache.h"
#include "rocksdb/advanced_cache.h"
#include "rocksdb/table.h"
#include "rocksdb/write_buffer_manager.h"
#include "rocksdb/options.h"
#include "rocksdb/perf_context.h"
#include "rocksdb/perf_level.h"
#include "rocksdb/slice.h"
#include "rocksdb/iterator.h"
#include "rocksdb/utilities/transaction.h"
#include "rocksdb/utilities/transaction_db.h"
#include "rocksdb/version.h"

namespace {

class WalSyncHookEnv;

class WalSyncHookFile : public rocksdb::WritableFileWrapper {
 public:
  WalSyncHookFile(std::unique_ptr<rocksdb::WritableFile> target,
                  WalSyncHookEnv* environment, std::string file_name,
                  bool is_wal_file);
  rocksdb::Status Sync() override;

 private:
  std::unique_ptr<rocksdb::WritableFile> owned_target_;
  WalSyncHookEnv* environment_;
  std::string file_name_;
  bool is_wal_file_;
};

class WalSyncHookEnv : public rocksdb::EnvWrapper {
 public:
  WalSyncHookEnv() : rocksdb::EnvWrapper(rocksdb::Env::Default()) {}

  rocksdb::Status NewWritableFile(
      const std::string& file_name,
      std::unique_ptr<rocksdb::WritableFile>* result,
      const rocksdb::EnvOptions& options) override {
    std::unique_ptr<rocksdb::WritableFile> opened_file;
    const auto status = this->target()->NewWritableFile(file_name, &opened_file, options);
    if (!status.ok()) {
      return status;
    }
    result->reset(new WalSyncHookFile(
        std::move(opened_file), this, file_name, is_wal_file(file_name)));
    return status;
  }

  void arm() { armed_.store(true, std::memory_order_release); }

  bool take_armed() { return armed_.exchange(false, std::memory_order_acq_rel); }

  static bool is_wal_file(std::string_view file_name) {
    const auto separator = file_name.find_last_of('/');
    const auto base_name = file_name.substr(
        separator == std::string_view::npos ? 0 : separator + 1);
    if (base_name.size() < 5 || base_name.substr(base_name.size() - 4) != ".log") {
      return false;
    }
    const auto digits = base_name.substr(0, base_name.size() - 4);
    return std::all_of(digits.begin(), digits.end(),
                       [](unsigned char character) {
                         return character >= '0' && character <= '9';
                       });
  }

 private:
  std::atomic<bool> armed_{false};
};

WalSyncHookFile::WalSyncHookFile(
    std::unique_ptr<rocksdb::WritableFile> target,
    WalSyncHookEnv* environment, std::string file_name, bool is_wal_file)
    : rocksdb::WritableFileWrapper(target.get()),
      owned_target_(std::move(target)),
      environment_(environment),
      file_name_(std::move(file_name)),
      is_wal_file_(is_wal_file) {}

rocksdb::Status WalSyncHookFile::Sync() {
  if (environment_->take_armed()) {
    const std::string message = is_wal_file_
        ? "R1B3B_HOOK|rocksdb|wal_sync_before|" + file_name_ + "\n"
        : "R1B3B_WRONG_SYNC_TARGET|rocksdb|" + file_name_ + "\n";
    const auto ignored = write(STDOUT_FILENO, message.data(), message.size());
    static_cast<void>(ignored);
    for (;;) {
      pause();
    }
  }
  return rocksdb::WritableFileWrapper::Sync();
}

WalSyncHookEnv& wal_sync_hook_env() {
  static WalSyncHookEnv environment;
  return environment;
}

bool wal_sync_hook_enabled() {
  const auto* value = std::getenv("DODB_R1B3B_ENABLE_WAL_SYNC_HOOK");
  return value != nullptr && std::string_view(value) == "1";
}

struct ReadIterator {
  rocksdb::TransactionDB* database;
  const rocksdb::Snapshot* snapshot;
  std::unique_ptr<rocksdb::Iterator> iterator;

  ~ReadIterator() {
    iterator.reset();
    database->ReleaseSnapshot(snapshot);
  }
};

char* copy_string(const std::string& value) {
  auto* output = static_cast<char*>(std::malloc(value.size() + 1));
  if (output == nullptr) {
    return nullptr;
  }
  std::memcpy(output, value.data(), value.size());
  output[value.size()] = '\0';
  return output;
}

void set_error(char** error, const rocksdb::Status& status) {
  if (error != nullptr) {
    *error = copy_string(status.ToString());
  }
}

int query_int_property(rocksdb::DB* database, const unsigned char* property,
                       size_t property_length, uint64_t* value,
                       char** property_text) {
  if (database == nullptr || property == nullptr || value == nullptr) {
    return -1;
  }
  const rocksdb::Slice property_slice(
      reinterpret_cast<const char*>(property), property_length);
  if (database->GetIntProperty(property_slice, value)) {
    return 0;
  }
  std::string fallback_value;
  if (database->GetProperty(property_slice, &fallback_value)) {
    if (property_text != nullptr) {
      *property_text = copy_string(fallback_value);
      if (*property_text == nullptr) {
        return -1;
      }
    }
    return 2;
  }
  return 1;
}

}

extern "C" {

struct DodbSharedConfig {
  size_t cache_bytes;
  size_t wbm_bytes;
  size_t write_buffer_bytes;
  int flush_threads;
  int compaction_threads;
  int max_open_files;
};

struct DodbSharedResources {
  DodbSharedConfig config;
  std::shared_ptr<rocksdb::Cache> cache;
  std::shared_ptr<rocksdb::WriteBufferManager> wbm;
};

struct DodbSharedSnapshot {
  uint64_t cache_capacity_bytes;
  uint64_t cache_usage_bytes;
  uint64_t cache_pinned_usage_bytes;
  uint64_t wbm_limit_bytes;
  uint64_t wbm_usage_bytes;
  uint64_t wbm_mutable_bytes;
  uint64_t wbm_cache_reservation_bytes;
  uint64_t flush_threads;
  uint64_t compaction_threads;
};

void* dodb_rocksdb_shared_new(const DodbSharedConfig* config, char** error) {
  try {
    if (config == nullptr || config->cache_bytes == 0 || config->wbm_bytes == 0 ||
        config->wbm_bytes > config->cache_bytes || config->write_buffer_bytes < 65536 ||
        config->write_buffer_bytes > config->wbm_bytes || config->flush_threads < 1 ||
        config->compaction_threads < 1 || config->max_open_files < 20 ||
        config->flush_threads > 64 || config->compaction_threads > 64) {
      set_error(error, rocksdb::Status::InvalidArgument("invalid shared-resource configuration"));
      return nullptr;
    }
    auto resources = std::make_unique<DodbSharedResources>();
    resources->config = *config;
    rocksdb::LRUCacheOptions cache_options;
    cache_options.capacity = config->cache_bytes;
    cache_options.num_shard_bits = 0;
    cache_options.strict_capacity_limit = false;
    resources->cache = cache_options.MakeSharedCache();
    resources->wbm = std::make_shared<rocksdb::WriteBufferManager>(
        config->wbm_bytes, resources->cache, false);
    auto* environment = rocksdb::Env::Default();
    environment->SetBackgroundThreads(config->flush_threads, rocksdb::Env::HIGH);
    environment->SetBackgroundThreads(config->compaction_threads, rocksdb::Env::LOW);
    return resources.release();
  } catch (const std::exception& exception) {
    set_error(error, rocksdb::Status::Aborted(exception.what()));
    return nullptr;
  }
}

void dodb_rocksdb_shared_delete(void* resources) {
  delete static_cast<DodbSharedResources*>(resources);
}

void dodb_rocksdb_shared_snapshot(void* resource_pointer, DodbSharedSnapshot* snapshot) {
  auto* resources = static_cast<DodbSharedResources*>(resource_pointer);
  auto* environment = rocksdb::Env::Default();
  *snapshot = {
      resources->cache->GetCapacity(), resources->cache->GetUsage(),
      resources->cache->GetPinnedUsage(), resources->wbm->buffer_size(),
      resources->wbm->memory_usage(), resources->wbm->mutable_memtable_memory_usage(),
      resources->wbm->dummy_entries_in_cache_usage(),
      static_cast<uint64_t>(environment->GetBackgroundThreads(rocksdb::Env::HIGH)),
      static_cast<uint64_t>(environment->GetBackgroundThreads(rocksdb::Env::LOW))};
}

void* dodb_rocksdb_open_shared(const char* path, void* resource_pointer, char** error) {
  try {
    auto* resources = static_cast<DodbSharedResources*>(resource_pointer);
    rocksdb::Options options;
    options.create_if_missing = true;
    options.compression = rocksdb::kNoCompression;
    options.env = rocksdb::Env::Default();
    options.write_buffer_manager = resources->wbm;
    options.write_buffer_size = resources->config.write_buffer_bytes;
    options.max_write_buffer_number = 2;
    options.max_write_buffer_size_to_maintain = 1;
    options.max_background_jobs = resources->config.flush_threads + resources->config.compaction_threads;
    options.max_background_flushes = resources->config.flush_threads;
    options.max_background_compactions = resources->config.compaction_threads;
    options.max_subcompactions = 1;
    options.max_open_files = resources->config.max_open_files;
    rocksdb::BlockBasedTableOptions table;
    table.block_cache = resources->cache;
    table.cache_index_and_filter_blocks = true;
    table.pin_l0_filter_and_index_blocks_in_cache = false;
    table.pin_top_level_index_and_filter = false;
    options.table_factory.reset(rocksdb::NewBlockBasedTableFactory(table));
    rocksdb::TransactionDB* database = nullptr;
    const auto status = rocksdb::TransactionDB::Open(
        options, rocksdb::TransactionDBOptions(), path, &database);
    if (!status.ok()) {
      set_error(error, status);
      return nullptr;
    }
    return database;
  } catch (const std::exception& exception) {
    set_error(error, rocksdb::Status::Aborted(exception.what()));
    return nullptr;
  }
}

bool dodb_rocksdb_uses_shared(void* database_pointer, void* resource_pointer) {
  auto* database = static_cast<rocksdb::TransactionDB*>(database_pointer);
  auto* resources = static_cast<DodbSharedResources*>(resource_pointer);
  const auto options = database->GetOptions();
  const auto* table = options.table_factory->GetOptions<rocksdb::BlockBasedTableOptions>();
  return options.write_buffer_manager == resources->wbm && table != nullptr &&
      table->block_cache == resources->cache && table->cache_index_and_filter_blocks &&
      !table->pin_l0_filter_and_index_blocks_in_cache &&
      !table->pin_top_level_index_and_filter && resources->wbm->cost_to_cache() &&
      options.env == rocksdb::Env::Default() &&
      options.write_buffer_size == resources->config.write_buffer_bytes &&
      options.max_open_files == resources->config.max_open_files &&
      options.max_background_flushes == resources->config.flush_threads &&
      options.max_background_compactions == resources->config.compaction_threads &&
      options.max_background_jobs == resources->config.flush_threads + resources->config.compaction_threads &&
      options.max_subcompactions == 1 && options.max_write_buffer_size_to_maintain == 1 &&
      options.max_write_buffer_number == 2 && options.compression == rocksdb::kNoCompression;
}

struct DodbRocksDbReadTimings {
  uint64_t native_call_nanos;
  uint64_t value_copy_nanos;
};

void* dodb_rocksdb_open(const char* path, char** error) {
  if (ROCKSDB_MAJOR != 11 || ROCKSDB_MINOR != 8 || ROCKSDB_PATCH != 1) {
    if (error != nullptr) {
      *error = copy_string("linked RocksDB is not version 11.8.1");
    }
    return nullptr;
  }
  rocksdb::Options options;
  options.create_if_missing = true;
  options.compression = rocksdb::kNoCompression;
  if (wal_sync_hook_enabled()) {
    options.env = &wal_sync_hook_env();
  }
  rocksdb::TransactionDBOptions transaction_options;
  rocksdb::TransactionDB* database = nullptr;
  const auto status = rocksdb::TransactionDB::Open(
      options, transaction_options, path, &database);
  if (!status.ok()) {
    set_error(error, status);
    return nullptr;
  }
  return database;
}

void dodb_rocksdb_arm_wal_sync_hook() {
  wal_sync_hook_env().arm();
}

void dodb_rocksdb_close(void* database) {
  delete static_cast<rocksdb::TransactionDB*>(database);
}

int dodb_rocksdb_get_int_property(void* database,
                                  const unsigned char* property,
                                  size_t property_length, uint64_t* value,
                                  char** property_text) {
  auto* database_handle = static_cast<rocksdb::TransactionDB*>(database);
  return query_int_property(database_handle, property, property_length, value,
                            property_text);
}

int dodb_rocksdb_get_int_property_for_db(
    void* database, const unsigned char* property, size_t property_length,
    uint64_t* value, char** property_text) {
  auto* database_handle = static_cast<rocksdb::DB*>(database);
  return query_int_property(database_handle, property, property_length, value,
                            property_text);
}

void* dodb_rocksdb_iterator_new(void* database, const unsigned char* start_key,
                                size_t start_key_length, char** error) {
  auto* database_handle = static_cast<rocksdb::TransactionDB*>(database);
  const auto* snapshot = database_handle->GetSnapshot();
  if (snapshot == nullptr) {
    if (error != nullptr) {
      *error = copy_string("RocksDB snapshot creation returned null");
    }
    return nullptr;
  }
  rocksdb::ReadOptions read_options;
  read_options.snapshot = snapshot;
  auto* read_iterator = new ReadIterator{
      database_handle, snapshot,
      std::unique_ptr<rocksdb::Iterator>(database_handle->NewIterator(read_options))};
  if (read_iterator->iterator == nullptr) {
    delete read_iterator;
    if (error != nullptr) {
      *error = copy_string("RocksDB iterator creation returned null");
    }
    return nullptr;
  }
  read_iterator->iterator->Seek(rocksdb::Slice(
      reinterpret_cast<const char*>(start_key), start_key_length));
  return read_iterator;
}

int dodb_rocksdb_iterator_valid(void* iterator) {
  return static_cast<ReadIterator*>(iterator)->iterator->Valid() ? 1 : 0;
}

const unsigned char* dodb_rocksdb_iterator_key(void* iterator, size_t* key_length) {
  const auto key = static_cast<ReadIterator*>(iterator)->iterator->key();
  *key_length = key.size();
  return reinterpret_cast<const unsigned char*>(key.data());
}

const unsigned char* dodb_rocksdb_iterator_value(void* iterator, size_t* value_length) {
  const auto value = static_cast<ReadIterator*>(iterator)->iterator->value();
  *value_length = value.size();
  return reinterpret_cast<const unsigned char*>(value.data());
}

int dodb_rocksdb_iterator_status(void* iterator, char** error) {
  const auto status = static_cast<ReadIterator*>(iterator)->iterator->status();
  if (status.ok()) {
    return 0;
  }
  set_error(error, status);
  return 1;
}

void dodb_rocksdb_iterator_next(void* iterator) {
  static_cast<ReadIterator*>(iterator)->iterator->Next();
}

void dodb_rocksdb_iterator_destroy(void* iterator) {
  delete static_cast<ReadIterator*>(iterator);
}

void* dodb_rocksdb_begin(void* database, char** error) {
  rocksdb::WriteOptions write_options;
  write_options.sync = true;
  write_options.disableWAL = false;
  auto* transaction = static_cast<rocksdb::TransactionDB*>(database)->BeginTransaction(write_options);
  if (transaction == nullptr && error != nullptr) {
    *error = copy_string("TransactionDB::BeginTransaction returned null");
  }
  return transaction;
}

int dodb_rocksdb_get(void* database, const unsigned char* key, size_t key_length,
                     unsigned char** value, size_t* value_length, char** error) {
  std::string stored;
  const auto status = static_cast<rocksdb::TransactionDB*>(database)->Get(
      rocksdb::ReadOptions(), rocksdb::Slice(reinterpret_cast<const char*>(key), key_length), &stored);
  if (status.IsNotFound()) {
    return 1;
  }
  if (!status.ok()) {
    set_error(error, status);
    return 2;
  }
  *value = reinterpret_cast<unsigned char*>(copy_string(stored));
  *value_length = stored.size();
  if (*value == nullptr) {
    if (error != nullptr) {
      *error = copy_string("allocation failed while copying RocksDB value");
    }
    return 2;
  }
  return 0;
}

int dodb_rocksdb_transaction_get_for_update(
    void* transaction, const unsigned char* key, size_t key_length,
    int exclusive, unsigned char** value, size_t* value_length,
    int collect_lock_wait,
    uint64_t* lock_wait_nanos, uint64_t* lock_wait_count,
    DodbRocksDbReadTimings* timings, char** error) {
  auto* perf_context = rocksdb::get_perf_context();
  if (collect_lock_wait != 0) {
    perf_context->Reset();
  }
  std::string stored;
  const auto native_started = collect_lock_wait != 0
      ? std::chrono::steady_clock::now() : std::chrono::steady_clock::time_point{};
  const auto status = static_cast<rocksdb::Transaction*>(transaction)->GetForUpdate(
      rocksdb::ReadOptions(), rocksdb::Slice(reinterpret_cast<const char*>(key), key_length), &stored,
      exclusive != 0, true);
  if (collect_lock_wait != 0) {
    timings->native_call_nanos = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - native_started).count();
    *lock_wait_nanos = perf_context->key_lock_wait_time;
    *lock_wait_count = perf_context->key_lock_wait_count;
  }
  if (status.IsNotFound()) {
    return 1;
  }
  if (!status.ok()) {
    set_error(error, status);
    return 2;
  }
  const auto copy_started = collect_lock_wait != 0
      ? std::chrono::steady_clock::now() : std::chrono::steady_clock::time_point{};
  *value = reinterpret_cast<unsigned char*>(copy_string(stored));
  *value_length = stored.size();
  if (collect_lock_wait != 0) {
    timings->value_copy_nanos = std::chrono::duration_cast<std::chrono::nanoseconds>(
        std::chrono::steady_clock::now() - copy_started).count();
  }
  if (*value == nullptr) {
    if (error != nullptr) {
      *error = copy_string("allocation failed while copying RocksDB value");
    }
    return 2;
  }
  return 0;
}

int dodb_rocksdb_transaction_lock_for_update(
    void* transaction, const unsigned char* key, size_t key_length,
    char** error) {
  const auto status = static_cast<rocksdb::Transaction*>(transaction)->GetForUpdate(
      rocksdb::ReadOptions(),
      rocksdb::Slice(reinterpret_cast<const char*>(key), key_length),
      static_cast<std::string*>(nullptr), true, true);
  if (!status.ok()) {
    set_error(error, status);
    return 1;
  }
  return 0;
}

void dodb_rocksdb_set_research_metrics(bool enabled) {
  rocksdb::SetPerfLevel(enabled ? rocksdb::PerfLevel::kEnableTimeExceptForMutex
                                : rocksdb::PerfLevel::kDisable);
  rocksdb::get_perf_context()->Reset();
}

int dodb_rocksdb_transaction_put(void* transaction, const unsigned char* key,
                                 size_t key_length, const unsigned char* value,
                                 size_t value_length, char** error) {
  const auto status = static_cast<rocksdb::Transaction*>(transaction)->Put(
      rocksdb::Slice(reinterpret_cast<const char*>(key), key_length),
      rocksdb::Slice(reinterpret_cast<const char*>(value), value_length));
  if (!status.ok()) {
    set_error(error, status);
    return 1;
  }
  return 0;
}

int dodb_rocksdb_transaction_commit(void* transaction, char** error) {
  const auto status = static_cast<rocksdb::Transaction*>(transaction)->Commit();
  if (!status.ok()) {
    set_error(error, status);
    return 1;
  }
  return 0;
}

void dodb_rocksdb_transaction_rollback(void* transaction) {
  auto* current = static_cast<rocksdb::Transaction*>(transaction);
  current->Rollback();
  delete current;
}

void dodb_rocksdb_transaction_destroy(void* transaction) {
  delete static_cast<rocksdb::Transaction*>(transaction);
}

void dodb_rocksdb_value_free(unsigned char* value) {
  std::free(value);
}

void dodb_rocksdb_error_free(char* error) {
  std::free(error);
}

}
