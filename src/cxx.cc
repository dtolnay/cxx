#include "../include/cxx.h"
#include <cassert>
#include <cstring>
#include <exception>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <type_traits>
#include <vector>

extern "C" {
const char *cxxbridge05$cxx_string$data(const std::string &s) noexcept {
  return s.data();
}

size_t cxxbridge05$cxx_string$length(const std::string &s) noexcept {
  return s.length();
}

// rust::String
void cxxbridge05$string$new(rust::String *self) noexcept;
void cxxbridge05$string$clone(rust::String *self,
                              const rust::String &other) noexcept;
bool cxxbridge05$string$from(rust::String *self, const char *ptr,
                             size_t len) noexcept;
void cxxbridge05$string$drop(rust::String *self) noexcept;
const char *cxxbridge05$string$ptr(const rust::String *self) noexcept;
size_t cxxbridge05$string$len(const rust::String *self) noexcept;

// rust::Str
bool cxxbridge05$str$valid(const char *ptr, size_t len) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge05 {

template <typename Exception>
void panic [[noreturn]] (const char *msg) {
#if defined(RUST_CXX_NO_EXCEPTIONS)
  std::cerr << "Error: " << msg << ". Aborting." << std::endl;
  std::terminate();
#else
  throw Exception(msg);
#endif
}

template void panic<std::out_of_range>[[noreturn]] (const char *msg);

String::String() noexcept { cxxbridge05$string$new(this); }

String::String(const String &other) noexcept {
  cxxbridge05$string$clone(this, other);
}

String::String(String &&other) noexcept {
  this->repr = other.repr;
  cxxbridge05$string$new(&other);
}

String::~String() noexcept { cxxbridge05$string$drop(this); }

static void initString(String *self, const char *s, size_t len) {
  if (!cxxbridge05$string$from(self, s, len)) {
    panic<std::invalid_argument>("data for rust::String is not utf-8");
  }
}

String::String(const std::string &s) { initString(this, s.data(), s.length()); }

String::String(const char *s) {
  assert(s != nullptr);
  initString(this, s, std::strlen(s));
}

String::String(const char *s, size_t len) {
  assert(s != nullptr || len == 0);
  initString(this,
             s == nullptr && len == 0 ? reinterpret_cast<const char *>(1) : s,
             len);
}

String &String::operator=(const String &other) noexcept {
  if (this != &other) {
    cxxbridge05$string$drop(this);
    cxxbridge05$string$clone(this, other);
  }
  return *this;
}

String &String::operator=(String &&other) noexcept {
  if (this != &other) {
    cxxbridge05$string$drop(this);
    this->repr = other.repr;
    cxxbridge05$string$new(&other);
  }
  return *this;
}

String::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *String::data() const noexcept {
  return cxxbridge05$string$ptr(this);
}

size_t String::size() const noexcept { return cxxbridge05$string$len(this); }

size_t String::length() const noexcept { return cxxbridge05$string$len(this); }

String::String(unsafe_bitcopy_t, const String &bits) noexcept
    : repr(bits.repr) {}

std::ostream &operator<<(std::ostream &os, const String &s) {
  os.write(s.data(), s.size());
  return os;
}

Str::Str() noexcept : ptr(reinterpret_cast<const char *>(1)), len(0) {}

static void initStr(const char *ptr, size_t len) {
  if (!cxxbridge05$str$valid(ptr, len)) {
    panic<std::invalid_argument>("data for rust::Str is not utf-8");
  }
}

Str::Str(const std::string &s) : ptr(s.data()), len(s.length()) {
  initStr(this->ptr, this->len);
}

Str::Str(const char *s) : ptr(s), len(std::strlen(s)) {
  assert(s != nullptr);
  initStr(this->ptr, this->len);
}

Str::Str(const char *s, size_t len)
    : ptr(s == nullptr && len == 0 ? reinterpret_cast<const char *>(1) : s),
      len(len) {
  assert(s != nullptr || len == 0);
  initStr(this->ptr, this->len);
}

Str::operator std::string() const {
  return std::string(this->data(), this->size());
}

std::ostream &operator<<(std::ostream &os, const Str &s) {
  os.write(s.data(), s.size());
  return os;
}

static_assert(std::is_trivially_copy_constructible<Str>::value,
              "trivial Str(const Str &)");
static_assert(std::is_trivially_copy_assignable<Str>::value,
              "trivial operator=(const Str &)");
static_assert(std::is_trivially_destructible<Str>::value, "trivial ~Str()");

extern "C" {
const char *cxxbridge05$error(const char *ptr, size_t len) {
  char *copy = new char[len];
  std::strncpy(copy, ptr, len);
  return copy;
}
} // extern "C"

Error::Error(const Error &other)
    : std::exception(other), msg(cxxbridge05$error(other.msg, other.len)),
      len(other.len) {}

Error::Error(Error &&other) noexcept
    : std::exception(std::move(other)), msg(other.msg), len(other.len) {
  other.msg = nullptr;
  other.len = 0;
}

Error::~Error() noexcept { delete[] this->msg; }

Error &Error::operator=(const Error &other) {
  if (this != &other) {
    std::exception::operator=(other);
    delete[] this->msg;
    this->msg = nullptr;
    this->msg = cxxbridge05$error(other.msg, other.len);
    this->len = other.len;
  }
  return *this;
}

Error &Error::operator=(Error &&other) noexcept {
  if (this != &other) {
    std::exception::operator=(std::move(other));
    this->msg = other.msg;
    this->len = other.len;
    other.msg = nullptr;
    other.len = 0;
  }
  return *this;
}

const char *Error::what() const noexcept { return this->msg; }

} // namespace cxxbridge05
} // namespace rust

extern "C" {
void cxxbridge05$unique_ptr$std$string$null(
    std::unique_ptr<std::string> *ptr) noexcept {
  new (ptr) std::unique_ptr<std::string>();
}
void cxxbridge05$unique_ptr$std$string$raw(std::unique_ptr<std::string> *ptr,
                                           std::string *raw) noexcept {
  new (ptr) std::unique_ptr<std::string>(raw);
}
const std::string *cxxbridge05$unique_ptr$std$string$get(
    const std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.get();
}
std::string *cxxbridge05$unique_ptr$std$string$release(
    std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge05$unique_ptr$std$string$drop(
    std::unique_ptr<std::string> *ptr) noexcept {
  ptr->~unique_ptr();
}
} // extern "C"

#define STD_VECTOR_OPS(RUST_TYPE, CXX_TYPE)                                    \
  size_t cxxbridge05$std$vector$##RUST_TYPE##$size(                            \
      const std::vector<CXX_TYPE> &s) noexcept {                               \
    return s.size();                                                           \
  }                                                                            \
  const CXX_TYPE *cxxbridge05$std$vector$##RUST_TYPE##$get_unchecked(          \
      const std::vector<CXX_TYPE> &s, size_t pos) noexcept {                   \
    return &s[pos];                                                            \
  }                                                                            \
  void cxxbridge05$unique_ptr$std$vector$##RUST_TYPE##$null(                   \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept {                  \
    new (ptr) std::unique_ptr<std::vector<CXX_TYPE>>();                        \
  }                                                                            \
  void cxxbridge05$unique_ptr$std$vector$##RUST_TYPE##$raw(                    \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr,                             \
      std::vector<CXX_TYPE> *raw) noexcept {                                   \
    new (ptr) std::unique_ptr<std::vector<CXX_TYPE>>(raw);                     \
  }                                                                            \
  const std::vector<CXX_TYPE>                                                  \
      *cxxbridge05$unique_ptr$std$vector$##RUST_TYPE##$get(                    \
          const std::unique_ptr<std::vector<CXX_TYPE>> &ptr) noexcept {        \
    return ptr.get();                                                          \
  }                                                                            \
  std::vector<CXX_TYPE>                                                        \
      *cxxbridge05$unique_ptr$std$vector$##RUST_TYPE##$release(                \
          std::unique_ptr<std::vector<CXX_TYPE>> &ptr) noexcept {              \
    return ptr.release();                                                      \
  }                                                                            \
  void cxxbridge05$unique_ptr$std$vector$##RUST_TYPE##$drop(                   \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept {                  \
    ptr->~unique_ptr();                                                        \
  }

#define RUST_VEC_EXTERNS(RUST_TYPE, CXX_TYPE)                                  \
  void cxxbridge05$rust_vec$##RUST_TYPE##$new(                                 \
      rust::Vec<CXX_TYPE> *ptr) noexcept;                                      \
  void cxxbridge05$rust_vec$##RUST_TYPE##$drop(                                \
      rust::Vec<CXX_TYPE> *ptr) noexcept;                                      \
  size_t cxxbridge05$rust_vec$##RUST_TYPE##$len(                               \
      const rust::Vec<CXX_TYPE> *ptr) noexcept;                                \
  const CXX_TYPE *cxxbridge05$rust_vec$##RUST_TYPE##$data(                     \
      const rust::Vec<CXX_TYPE> *ptr) noexcept;                                \
  void cxxbridge05$rust_vec$##RUST_TYPE##$reserve_total(                       \
      rust::Vec<CXX_TYPE> *ptr, size_t cap) noexcept;                          \
  void cxxbridge05$rust_vec$##RUST_TYPE##$set_len(rust::Vec<CXX_TYPE> *ptr,    \
                                                  size_t len) noexcept;        \
  size_t cxxbridge05$rust_vec$##RUST_TYPE##$stride() noexcept;

#define RUST_VEC_OPS(RUST_TYPE, CXX_TYPE)                                      \
  template <>                                                                  \
  Vec<CXX_TYPE>::Vec() noexcept {                                              \
    cxxbridge05$rust_vec$##RUST_TYPE##$new(this);                              \
  }                                                                            \
  template <>                                                                  \
  void Vec<CXX_TYPE>::drop() noexcept {                                        \
    return cxxbridge05$rust_vec$##RUST_TYPE##$drop(this);                      \
  }                                                                            \
  template <>                                                                  \
  size_t Vec<CXX_TYPE>::size() const noexcept {                                \
    return cxxbridge05$rust_vec$##RUST_TYPE##$len(this);                       \
  }                                                                            \
  template <>                                                                  \
  const CXX_TYPE *Vec<CXX_TYPE>::data() const noexcept {                       \
    return cxxbridge05$rust_vec$##RUST_TYPE##$data(this);                      \
  }                                                                            \
  template <>                                                                  \
  void Vec<CXX_TYPE>::reserve_total(size_t cap) noexcept {                     \
    cxxbridge05$rust_vec$##RUST_TYPE##$reserve_total(this, cap);               \
  }                                                                            \
  template <>                                                                  \
  void Vec<CXX_TYPE>::set_len(size_t len) noexcept {                           \
    cxxbridge05$rust_vec$##RUST_TYPE##$set_len(this, len);                     \
  }                                                                            \
  template <>                                                                  \
  size_t Vec<CXX_TYPE>::stride() noexcept {                                    \
    return cxxbridge05$rust_vec$##RUST_TYPE##$stride();                        \
  }

// Usize and isize are the same type as one of the below.
#define FOR_EACH_NUMERIC(MACRO)                                                \
  MACRO(u8, uint8_t)                                                           \
  MACRO(u16, uint16_t)                                                         \
  MACRO(u32, uint32_t)                                                         \
  MACRO(u64, uint64_t)                                                         \
  MACRO(i8, int8_t)                                                            \
  MACRO(i16, int16_t)                                                          \
  MACRO(i32, int32_t)                                                          \
  MACRO(i64, int64_t)                                                          \
  MACRO(f32, float)                                                            \
  MACRO(f64, double)

#define FOR_EACH_STD_VECTOR(MACRO)                                             \
  FOR_EACH_NUMERIC(MACRO)                                                      \
  MACRO(usize, size_t)                                                         \
  MACRO(isize, rust::isize)                                                    \
  MACRO(string, std::string)

#define FOR_EACH_RUST_VEC(MACRO)                                               \
  FOR_EACH_NUMERIC(MACRO)                                                      \
  MACRO(bool, bool)                                                            \
  MACRO(string, rust::String)

extern "C" {
FOR_EACH_STD_VECTOR(STD_VECTOR_OPS)
FOR_EACH_RUST_VEC(RUST_VEC_EXTERNS)
} // extern "C"

namespace rust {
inline namespace cxxbridge05 {
FOR_EACH_RUST_VEC(RUST_VEC_OPS)
} // namespace cxxbridge05
} // namespace rust
