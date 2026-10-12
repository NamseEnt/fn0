#include <cstdlib>
#include <cstring>
#include <memory>
#include <string>

#include "rocksdb/db.h"
#include "rocksdb/options.h"
#include "rocksdb/slice.h"
#include "rocksdb/iterator.h"
#include "rocksdb/utilities/transaction.h"
#include "rocksdb/utilities/transaction_db.h"
#include "rocksdb/version.h"

namespace {

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
