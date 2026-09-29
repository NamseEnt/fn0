#include <chrono>
#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <map>
#include <memory>
#include <mutex>
#include <sstream>
#include <string>
#include <vector>

#include "rocksdb/cache.h"
#include "rocksdb/db.h"
#include "rocksdb/listener.h"
#include "rocksdb/options.h"
#include "rocksdb/table.h"
#include "rocksdb/version.h"
#include "rocksdb/write_batch.h"

namespace {

std::string json_escape(const std::string& input) {
  std::string output;
  output.reserve(input.size() + 8);
  for (char character : input) {
    switch (character) {
      case '"':
        output += "\\\"";
        break;
      case '\\':
        output += "\\\\";
        break;
      case '\n':
        output += "\\n";
        break;
      case '\t':
        output += "\\t";
        break;
      default:
        if (static_cast<unsigned char>(character) < 0x20) {
          output += ' ';
        } else {
          output += character;
        }
    }
  }
  return output;
}

double monotonic_seconds() {
  return std::chrono::duration<double>(
             std::chrono::steady_clock::now().time_since_epoch())
      .count();
}

const char* stall_name(rocksdb::WriteStallCondition condition) {
  switch (condition) {
    case rocksdb::WriteStallCondition::kNormal:
      return "normal";
    case rocksdb::WriteStallCondition::kDelayed:
      return "delayed";
    case rocksdb::WriteStallCondition::kStopped:
      return "stopped";
  }
  return "unknown";
}

class EventRecorder : public rocksdb::EventListener {
 public:
  void OnFlushCompleted(rocksdb::DB*,
                        const rocksdb::FlushJobInfo& info) override {
    std::ostringstream event;
    event << "{\"type\":\"flush\",\"t_mono\":" << monotonic_seconds()
          << ",\"file_number\":" << info.file_number
          << ",\"data_size\":" << info.table_properties.data_size
          << ",\"num_entries\":" << info.table_properties.num_entries
          << ",\"triggered_writes_slowdown\":"
          << (info.triggered_writes_slowdown ? "true" : "false")
          << ",\"triggered_writes_stop\":"
          << (info.triggered_writes_stop ? "true" : "false")
          << ",\"flush_reason\":" << static_cast<int>(info.flush_reason)
          << "}";
    push(event.str());
  }

  void OnCompactionCompleted(rocksdb::DB*,
                             const rocksdb::CompactionJobInfo& info) override {
    std::ostringstream event;
    event << "{\"type\":\"compaction\",\"t_mono\":" << monotonic_seconds()
          << ",\"base_input_level\":" << info.base_input_level
          << ",\"output_level\":" << info.output_level
          << ",\"input_files\":" << info.input_files.size()
          << ",\"output_files\":" << info.output_files.size()
          << ",\"total_input_bytes\":" << info.stats.total_input_bytes
          << ",\"total_output_bytes\":" << info.stats.total_output_bytes
          << ",\"elapsed_micros\":" << info.stats.elapsed_micros
          << ",\"reason\":" << static_cast<int>(info.compaction_reason)
          << ",\"status_ok\":" << (info.status.ok() ? "true" : "false")
          << "}";
    push(event.str());
  }

  void OnStallConditionsChanged(const rocksdb::WriteStallInfo& info) override {
    std::ostringstream event;
    event << "{\"type\":\"stall\",\"t_mono\":" << monotonic_seconds()
          << ",\"cf\":\"" << json_escape(info.cf_name) << "\",\"current\":\""
          << stall_name(info.condition.cur) << "\",\"previous\":\""
          << stall_name(info.condition.prev) << "\"}";
    push(event.str());
  }

  std::string drain_json() {
    std::lock_guard<std::mutex> guard(mutex_);
    std::string output = "[";
    for (size_t event_index = 0; event_index < events_.size(); ++event_index) {
      if (event_index > 0) {
        output += ",";
      }
      output += events_[event_index];
    }
    output += "]";
    events_.clear();
    return output;
  }

 private:
  void push(std::string event) {
    std::lock_guard<std::mutex> guard(mutex_);
    events_.push_back(std::move(event));
  }

  std::mutex mutex_;
  std::vector<std::string> events_;
};

struct Handle {
  std::unique_ptr<rocksdb::DB> db;
  std::shared_ptr<EventRecorder> recorder;
  rocksdb::WriteOptions write_options;
  std::string options_summary;
};

char* copy_string(const std::string& value) {
  char* output = static_cast<char*>(std::malloc(value.size() + 1));
  std::memcpy(output, value.data(), value.size());
  output[value.size()] = '\0';
  return output;
}

}

extern "C" {

void* crossdb_rocksdb_open(const char* path, int pipelined,
                           uint64_t block_cache_bytes, char** error) {
  rocksdb::Options options;
  options.create_if_missing = true;
  options.compression = rocksdb::kNoCompression;
  options.enable_pipelined_write = pipelined != 0;
  rocksdb::BlockBasedTableOptions table_options;
  table_options.block_size = 4096;
  table_options.block_cache = rocksdb::NewLRUCache(block_cache_bytes);
  options.table_factory.reset(rocksdb::NewBlockBasedTableFactory(table_options));
  auto recorder = std::make_shared<EventRecorder>();
  options.listeners.push_back(recorder);

  std::unique_ptr<rocksdb::DB> db;
  rocksdb::Status status = rocksdb::DB::Open(options, path, &db);
  if (!status.ok()) {
    *error = copy_string(status.ToString());
    return nullptr;
  }
  auto* handle = new Handle();
  handle->db = std::move(db);
  handle->recorder = recorder;
  handle->write_options.sync = true;
  handle->write_options.disableWAL = false;
  std::ostringstream summary;
  summary << "{\"rocksdb_version\":\"" << ROCKSDB_MAJOR << "." << ROCKSDB_MINOR
          << "." << ROCKSDB_PATCH << "\""
          << ",\"compression\":\"kNoCompression\""
          << ",\"block_size\":" << table_options.block_size
          << ",\"block_cache_bytes\":" << block_cache_bytes
          << ",\"enable_pipelined_write\":"
          << (options.enable_pipelined_write ? "true" : "false")
          << ",\"use_fsync\":" << (options.use_fsync ? "true" : "false")
          << ",\"write_buffer_size\":" << options.write_buffer_size
          << ",\"max_write_buffer_number\":" << options.max_write_buffer_number
          << ",\"level0_file_num_compaction_trigger\":"
          << options.level0_file_num_compaction_trigger
          << ",\"level0_slowdown_writes_trigger\":"
          << options.level0_slowdown_writes_trigger
          << ",\"level0_stop_writes_trigger\":"
          << options.level0_stop_writes_trigger
          << ",\"max_background_jobs\":" << options.max_background_jobs
          << ",\"allow_concurrent_memtable_write\":"
          << (options.allow_concurrent_memtable_write ? "true" : "false")
          << ",\"enable_write_thread_adaptive_yield\":"
          << (options.enable_write_thread_adaptive_yield ? "true" : "false")
          << ",\"manual_wal_flush\":"
          << (options.manual_wal_flush ? "true" : "false")
          << ",\"write_options_sync\":"
          << (handle->write_options.sync ? "true" : "false")
          << ",\"write_options_disable_wal\":"
          << (handle->write_options.disableWAL ? "true" : "false") << "}";
  handle->options_summary = summary.str();
  return handle;
}

int crossdb_rocksdb_write(void* raw_handle, const uint8_t* keys,
                          const uint8_t* values, size_t count, size_t key_length,
                          size_t value_length, char** error) {
  auto* handle = static_cast<Handle*>(raw_handle);
  rocksdb::WriteBatch batch;
  for (size_t mutation_index = 0; mutation_index < count; ++mutation_index) {
    batch.Put(rocksdb::Slice(reinterpret_cast<const char*>(keys) +
                                 mutation_index * key_length,
                             key_length),
              rocksdb::Slice(reinterpret_cast<const char*>(values) +
                                 mutation_index * value_length,
                             value_length));
  }
  rocksdb::Status status = handle->db->Write(handle->write_options, &batch);
  if (!status.ok()) {
    *error = copy_string(status.ToString());
    return 1;
  }
  return 0;
}

int crossdb_rocksdb_get(void* raw_handle, const uint8_t* key, size_t key_length,
                        char** value, size_t* value_length) {
  auto* handle = static_cast<Handle*>(raw_handle);
  std::string output;
  rocksdb::Status status = handle->db->Get(
      rocksdb::ReadOptions(),
      rocksdb::Slice(reinterpret_cast<const char*>(key), key_length), &output);
  if (status.IsNotFound()) {
    return 1;
  }
  if (!status.ok()) {
    return 2;
  }
  *value = copy_string(output);
  *value_length = output.size();
  return 0;
}

uint64_t crossdb_rocksdb_count(void* raw_handle) {
  auto* handle = static_cast<Handle*>(raw_handle);
  std::unique_ptr<rocksdb::Iterator> iterator(
      handle->db->NewIterator(rocksdb::ReadOptions()));
  uint64_t count = 0;
  for (iterator->SeekToFirst(); iterator->Valid(); iterator->Next()) {
    ++count;
  }
  return count;
}

char* crossdb_rocksdb_property(void* raw_handle, const char* name) {
  auto* handle = static_cast<Handle*>(raw_handle);
  std::string value;
  if (!handle->db->GetProperty(name, &value)) {
    return nullptr;
  }
  return copy_string(value);
}

char* crossdb_rocksdb_map_property_json(void* raw_handle, const char* name) {
  auto* handle = static_cast<Handle*>(raw_handle);
  std::map<std::string, std::string> values;
  if (!handle->db->GetMapProperty(name, &values)) {
    return nullptr;
  }
  std::string output = "{";
  bool first = true;
  for (const auto& entry : values) {
    if (!first) {
      output += ",";
    }
    first = false;
    output += "\"" + json_escape(entry.first) + "\":\"" +
              json_escape(entry.second) + "\"";
  }
  output += "}";
  return copy_string(output);
}

char* crossdb_rocksdb_events(void* raw_handle) {
  auto* handle = static_cast<Handle*>(raw_handle);
  return copy_string(handle->recorder->drain_json());
}

char* crossdb_rocksdb_options(void* raw_handle) {
  auto* handle = static_cast<Handle*>(raw_handle);
  return copy_string(handle->options_summary);
}

double crossdb_monotonic_seconds() { return monotonic_seconds(); }

char* crossdb_rocksdb_close(void* raw_handle) {
  auto* handle = static_cast<Handle*>(raw_handle);
  rocksdb::Status status = handle->db->Close();
  std::string result = status.ok() ? "ok" : status.ToString();
  delete handle;
  return copy_string(result);
}

void crossdb_free(char* value) { std::free(value); }
}
