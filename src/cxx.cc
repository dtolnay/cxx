#include "../include/cxx.h"
#include <cstring>
#include <exception>
#include <iostream>
#include <memory>
#include <vector>
#include <stdexcept>

template <typename Exception>
static void panic [[noreturn]] (const char *msg) {
#if defined(RUST_CXX_NO_EXCEPTIONS)
  std::cerr << "Error: " << msg << ". Aborting." << std::endl;
  std::terminate();
#else
  throw Exception(msg);
#endif
}

extern "C" {
const char *cxxbridge02$cxx_string$data(const std::string &s) noexcept {
  return s.data();
}

size_t cxxbridge02$cxx_string$length(const std::string &s) noexcept {
  return s.length();
}

// rust::String
void cxxbridge02$string$new(rust::String *self) noexcept;
void cxxbridge02$string$clone(rust::String *self,
                              const rust::String &other) noexcept;
bool cxxbridge02$string$from(rust::String *self, const char *ptr,
                             size_t len) noexcept;
void cxxbridge02$string$drop(rust::String *self) noexcept;
const char *cxxbridge02$string$ptr(const rust::String *self) noexcept;
size_t cxxbridge02$string$len(const rust::String *self) noexcept;

// rust::Str
bool cxxbridge02$str$valid(const char *ptr, size_t len) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge02 {

String::String() noexcept { cxxbridge02$string$new(this); }

String::String(const String &other) noexcept {
  cxxbridge02$string$clone(this, other);
}

String::String(String &&other) noexcept {
  this->repr = other.repr;
  cxxbridge02$string$new(&other);
}

String::~String() noexcept { cxxbridge02$string$drop(this); }

String::String(const std::string &s) {
  auto ptr = s.data();
  auto len = s.length();
  if (!cxxbridge02$string$from(this, ptr, len)) {
    panic<std::invalid_argument>("data for rust::String is not utf-8");
  }
}

String::String(const char *s) {
  auto len = std::strlen(s);
  if (!cxxbridge02$string$from(this, s, len)) {
    panic<std::invalid_argument>("data for rust::String is not utf-8");
  }
}

String &String::operator=(const String &other) noexcept {
  if (this != &other) {
    cxxbridge02$string$drop(this);
    cxxbridge02$string$clone(this, other);
  }
  return *this;
}

String &String::operator=(String &&other) noexcept {
  if (this != &other) {
    cxxbridge02$string$drop(this);
    this->repr = other.repr;
    cxxbridge02$string$new(&other);
  }
  return *this;
}

String::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *String::data() const noexcept {
  return cxxbridge02$string$ptr(this);
}

size_t String::size() const noexcept { return cxxbridge02$string$len(this); }

size_t String::length() const noexcept { return cxxbridge02$string$len(this); }

String::String(unsafe_bitcopy_t, const String &bits) noexcept
    : repr(bits.repr) {}

std::ostream &operator<<(std::ostream &os, const String &s) {
  os.write(s.data(), s.size());
  return os;
}

Str::Str() noexcept : repr(Repr{reinterpret_cast<const char *>(this), 0}) {}

Str::Str(const Str &) noexcept = default;

Str::Str(const std::string &s) : repr(Repr{s.data(), s.length()}) {
  if (!cxxbridge02$str$valid(this->repr.ptr, this->repr.len)) {
    panic<std::invalid_argument>("data for rust::Str is not utf-8");
  }
}

Str::Str(const char *s) : repr(Repr{s, std::strlen(s)}) {
  if (!cxxbridge02$str$valid(this->repr.ptr, this->repr.len)) {
    panic<std::invalid_argument>("data for rust::Str is not utf-8");
  }
}

Str &Str::operator=(Str other) noexcept {
  this->repr = other.repr;
  return *this;
}

Str::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *Str::data() const noexcept { return this->repr.ptr; }

size_t Str::size() const noexcept { return this->repr.len; }

size_t Str::length() const noexcept { return this->repr.len; }

Str::Str(Repr repr_) noexcept : repr(repr_) {}

Str::operator Repr() noexcept { return this->repr; }

std::ostream &operator<<(std::ostream &os, const Str &s) {
  os.write(s.data(), s.size());
  return os;
}

extern "C" {
const char *cxxbridge02$error(const char *ptr, size_t len) {
  char *copy = new char[len];
  strncpy(copy, ptr, len);
  return copy;
}
} // extern "C"

Error::Error(Str::Repr msg) noexcept : msg(msg) {}

Error::Error(const Error &other) {
  this->msg.ptr = cxxbridge02$error(other.msg.ptr, other.msg.len);
  this->msg.len = other.msg.len;
}

Error::Error(Error &&other) noexcept {
  delete[] this->msg.ptr;
  this->msg = other.msg;
  other.msg.ptr = nullptr;
  other.msg.len = 0;
}

Error::~Error() noexcept { delete[] this->msg.ptr; }

const char *Error::what() const noexcept { return this->msg.ptr; }

} // namespace cxxbridge02
} // namespace rust

extern "C" {
void cxxbridge02$unique_ptr$std$string$null(
    std::unique_ptr<std::string> *ptr) noexcept {
  new (ptr) std::unique_ptr<std::string>();
}
void cxxbridge02$unique_ptr$std$string$raw(std::unique_ptr<std::string> *ptr,
                                           std::string *raw) noexcept {
  new (ptr) std::unique_ptr<std::string>(raw);
}
const std::string *cxxbridge02$unique_ptr$std$string$get(
    const std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.get();
}
std::string *cxxbridge02$unique_ptr$std$string$release(
    std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge02$unique_ptr$std$string$drop(
    std::unique_ptr<std::string> *ptr) noexcept {
  ptr->~unique_ptr();
}
} // extern "C"

#define STD_VECTOR_OPS(RUST_TYPE, CXX_TYPE) \
extern "C" { \
size_t cxxbridge02$std$vector$##RUST_TYPE##$length(const std::vector<CXX_TYPE> &s) noexcept { \
  return s.size(); \
} \
void cxxbridge02$std$vector$##RUST_TYPE##$push_back(std::vector<CXX_TYPE> &s, const CXX_TYPE &item) noexcept { \
  s.push_back(item); \
} \
const CXX_TYPE *cxxbridge02$std$vector$##RUST_TYPE##$get_unchecked(const std::vector<CXX_TYPE> &s, size_t pos) noexcept { \
  return &s[pos]; \
} \
static_assert(sizeof(::std::unique_ptr<std::vector<CXX_TYPE>>) == sizeof(void *), ""); \
static_assert(alignof(::std::unique_ptr<std::vector<CXX_TYPE>>) == alignof(void *), ""); \
void cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$null(::std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept { \
  new (ptr) ::std::unique_ptr<std::vector<CXX_TYPE>>(); \
} \
void cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$new(::std::unique_ptr<std::vector<CXX_TYPE>> *ptr, std::vector<CXX_TYPE> *value) noexcept { \
  new (ptr) ::std::unique_ptr<std::vector<CXX_TYPE>>(new std::vector<CXX_TYPE>(::std::move(*value))); \
} \
void cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$raw(::std::unique_ptr<std::vector<CXX_TYPE>> *ptr, std::vector<CXX_TYPE> *raw) noexcept { \
  new (ptr) ::std::unique_ptr<std::vector<CXX_TYPE>>(raw); \
} \
const std::vector<CXX_TYPE> *cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$get(const ::std::unique_ptr<std::vector<CXX_TYPE>>& ptr) noexcept { \
  return ptr.get(); \
} \
std::vector<CXX_TYPE> *cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$release(::std::unique_ptr<std::vector<CXX_TYPE>>& ptr) noexcept { \
  return ptr.release(); \
} \
void cxxbridge02$unique_ptr$std$vector$##RUST_TYPE##$drop(::std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept { \
  ptr->~unique_ptr(); \
} \
} // extern "C"

STD_VECTOR_OPS(u8, uint8_t);
STD_VECTOR_OPS(u16, uint16_t);
STD_VECTOR_OPS(u32, uint32_t);
STD_VECTOR_OPS(u64, uint64_t);
STD_VECTOR_OPS(usize, size_t);
STD_VECTOR_OPS(i8, int8_t);
STD_VECTOR_OPS(i16, int16_t);
STD_VECTOR_OPS(i32, int32_t);
STD_VECTOR_OPS(i64, int64_t);
STD_VECTOR_OPS(isize, rust::isize);
STD_VECTOR_OPS(f32, float);
STD_VECTOR_OPS(f64, double);

