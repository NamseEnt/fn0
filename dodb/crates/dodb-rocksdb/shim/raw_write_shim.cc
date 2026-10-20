#include <cstdlib>
#include <cstring>
#include <memory>

#include "rocksdb/db.h"
#include "rocksdb/iterator.h"
#include "rocksdb/options.h"
#include "rocksdb/slice.h"
#include "rocksdb/write_batch.h"

namespace {

char* copy_error(const rocksdb::Status& status) {
  const auto message = status.ToString();
  auto* result = static_cast<char*>(std::malloc(message.size() + 1));
  if (result == nullptr) {
    return nullptr;
  }
  std::memcpy(result, message.data(), message.size());
  result[message.size()] = '\0';
  return result;
}

}

extern "C" {

void* dodb_raw_rocksdb_open(const char* path, char** error) {
  rocksdb::Options options;
  options.create_if_missing = true;
  options.compression = rocksdb::kNoCompression;
  std::unique_ptr<rocksdb::DB> database;
  const auto status = rocksdb::DB::Open(options, path, &database);
  if (!status.ok()) {
    if (error != nullptr) {
      *error = copy_error(status);
    }
    return nullptr;
  }
  return database.release();
}

void dodb_raw_rocksdb_close(void* database) {
  delete static_cast<rocksdb::DB*>(database);
}

int dodb_raw_rocksdb_write_batch(
    void* database, const unsigned char* const* keys, const size_t* key_lengths,
    const unsigned char* const* values, const size_t* value_lengths,
    size_t count, char** error) {
  rocksdb::WriteBatch batch;
  for (size_t entry_index = 0; entry_index < count; ++entry_index) {
    batch.Put(rocksdb::Slice(reinterpret_cast<const char*>(keys[entry_index]), key_lengths[entry_index]),
              rocksdb::Slice(reinterpret_cast<const char*>(values[entry_index]), value_lengths[entry_index]));
  }
  rocksdb::WriteOptions write_options;
  write_options.sync = true;
  write_options.disableWAL = false;
  const auto status = static_cast<rocksdb::DB*>(database)->Write(write_options, &batch);
  if (!status.ok()) {
    if (error != nullptr) {
      *error = copy_error(status);
    }
    return 1;
  }
  return 0;
}

void* dodb_raw_rocksdb_iterator_new(void* database, char** error) {
  auto* iterator = static_cast<rocksdb::DB*>(database)->NewIterator(rocksdb::ReadOptions());
  if (iterator == nullptr && error != nullptr) {
    *error = copy_error(rocksdb::Status::IOError("iterator allocation returned null"));
  }
  if (iterator != nullptr) {
    iterator->SeekToFirst();
  }
  return iterator;
}

void dodb_raw_rocksdb_iterator_seek(void* iterator, const unsigned char* key,
                                   size_t key_length) {
  static_cast<rocksdb::Iterator*>(iterator)->Seek(
      rocksdb::Slice(reinterpret_cast<const char*>(key), key_length));
}

int dodb_raw_rocksdb_iterator_valid(void* iterator) {
  return static_cast<rocksdb::Iterator*>(iterator)->Valid() ? 1 : 0;
}

const unsigned char* dodb_raw_rocksdb_iterator_key(void* iterator, size_t* key_length) {
  const auto key = static_cast<rocksdb::Iterator*>(iterator)->key();
  *key_length = key.size();
  return reinterpret_cast<const unsigned char*>(key.data());
}

const unsigned char* dodb_raw_rocksdb_iterator_value(void* iterator, size_t* value_length) {
  const auto value = static_cast<rocksdb::Iterator*>(iterator)->value();
  *value_length = value.size();
  return reinterpret_cast<const unsigned char*>(value.data());
}

int dodb_raw_rocksdb_iterator_status(void* iterator, char** error) {
  const auto status = static_cast<rocksdb::Iterator*>(iterator)->status();
  if (status.ok()) {
    return 0;
  }
  if (error != nullptr) {
    *error = copy_error(status);
  }
  return 1;
}

void dodb_raw_rocksdb_iterator_next(void* iterator) {
  static_cast<rocksdb::Iterator*>(iterator)->Next();
}

void dodb_raw_rocksdb_iterator_destroy(void* iterator) {
  delete static_cast<rocksdb::Iterator*>(iterator);
}

void dodb_raw_rocksdb_free_error(char* error) {
  std::free(error);
}

}
