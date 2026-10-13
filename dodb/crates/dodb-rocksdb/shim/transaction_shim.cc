#include <atomic>
#include <algorithm>
#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>
#include <string_view>
#include <signal.h>
#include <unistd.h>

#include "rocksdb/db.h"
#include "rocksdb/env.h"
#include "rocksdb/options.h"
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

}

extern "C" {

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
    unsigned char** value, size_t* value_length, char** error) {
  std::string stored;
  const auto status = static_cast<rocksdb::Transaction*>(transaction)->GetForUpdate(
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
