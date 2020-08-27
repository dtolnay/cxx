#include "../include/cxx.h"
#include <cstring>
#include <exception>
#include <iostream>
#include <memory>
#include <stdexcept>
#include <vector>

extern "C" {
const char *cxxbridge03$cxx_string$data(const std::string &s) noexcept {
  return s.data();
}

size_t cxxbridge03$cxx_string$length(const std::string &s) noexcept {
  return s.length();
}

// rust::String
void cxxbridge03$string$new(rust::String *self) noexcept;
void cxxbridge03$string$clone(rust::String *self,
                              const rust::String &other) noexcept;
bool cxxbridge03$string$from(rust::String *self, const char *ptr,
                             size_t len) noexcept;
void cxxbridge03$string$drop(rust::String *self) noexcept;
const char *cxxbridge03$string$ptr(const rust::String *self) noexcept;
size_t cxxbridge03$string$len(const rust::String *self) noexcept;

// rust::Str
bool cxxbridge03$str$valid(const char *ptr, size_t len) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge03 {

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

String::String() noexcept { cxxbridge03$string$new(this); }

String::String(const String &other) noexcept {
  cxxbridge03$string$clone(this, other);
}

String::String(String &&other) noexcept {
  this->repr = other.repr;
  cxxbridge03$string$new(&other);
}

String::~String() noexcept { cxxbridge03$string$drop(this); }

String::String(const std::string &s) : String(s.data(), s.length()) {}

String::String(const char *s) : String(s, std::strlen(s)) {}

String::String(const char *s, size_t len) {
  if (!cxxbridge03$string$from(this, s, len)) {
    panic<std::invalid_argument>("data for rust::String is not utf-8");
  }
}

String &String::operator=(const String &other) noexcept {
  if (this != &other) {
    cxxbridge03$string$drop(this);
    cxxbridge03$string$clone(this, other);
  }
  return *this;
}

String &String::operator=(String &&other) noexcept {
  if (this != &other) {
    cxxbridge03$string$drop(this);
    this->repr = other.repr;
    cxxbridge03$string$new(&other);
  }
  return *this;
}

String::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *String::data() const noexcept {
  return cxxbridge03$string$ptr(this);
}

size_t String::size() const noexcept { return cxxbridge03$string$len(this); }

size_t String::length() const noexcept { return cxxbridge03$string$len(this); }

String::String(unsafe_bitcopy_t, const String &bits) noexcept
    : repr(bits.repr) {}

std::ostream &operator<<(std::ostream &os, const String &s) {
  os.write(s.data(), s.size());
  return os;
}

Str::Str() noexcept : repr(Repr{reinterpret_cast<const char *>(this), 0}) {}

Str::Str(const Str &) noexcept = default;

Str::Str(const std::string &s) : Str(s.data(), s.length()) {}

Str::Str(const char *s) : Str(s, std::strlen(s)) {}

Str::Str(const char *s, size_t len) : repr(Repr{s, len}) {
  if (!cxxbridge03$str$valid(this->repr.ptr, this->repr.len)) {
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
const char *cxxbridge03$error(const char *ptr, size_t len) {
  char *copy = new char[len];
  strncpy(copy, ptr, len);
  return copy;
}
} // extern "C"

Error::Error(Str::Repr msg) noexcept : msg(msg) {}

Error::Error(const Error &other) {
  this->msg.ptr = cxxbridge03$error(other.msg.ptr, other.msg.len);
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

} // namespace cxxbridge03
} // namespace rust

extern "C" {
void cxxbridge03$unique_ptr$std$string$null(
    std::unique_ptr<std::string> *ptr) noexcept {
  new (ptr) std::unique_ptr<std::string>();
}
void cxxbridge03$unique_ptr$std$string$raw(std::unique_ptr<std::string> *ptr,
                                           std::string *raw) noexcept {
  new (ptr) std::unique_ptr<std::string>(raw);
}
const std::string *cxxbridge03$unique_ptr$std$string$get(
    const std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.get();
}
std::string *cxxbridge03$unique_ptr$std$string$release(
    std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge03$unique_ptr$std$string$drop(
    std::unique_ptr<std::string> *ptr) noexcept {
  ptr->~unique_ptr();
}
} // extern "C"

#define STD_VECTOR_OPS(RUST_TYPE, CXX_TYPE)                                    \
  size_t cxxbridge03$std$vector$##RUST_TYPE##$size(                            \
      const std::vector<CXX_TYPE> &s) noexcept {                               \
    return s.size();                                                           \
  }                                                                            \
  const CXX_TYPE *cxxbridge03$std$vector$##RUST_TYPE##$get_unchecked(          \
      const std::vector<CXX_TYPE> &s, size_t pos) noexcept {                   \
    return &s[pos];                                                            \
  }                                                                            \
  void cxxbridge03$unique_ptr$std$vector$##RUST_TYPE##$null(                   \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept {                  \
    new (ptr) std::unique_ptr<std::vector<CXX_TYPE>>();                        \
  }                                                                            \
  void cxxbridge03$unique_ptr$std$vector$##RUST_TYPE##$raw(                    \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr,                             \
      std::vector<CXX_TYPE> *raw) noexcept {                                   \
    new (ptr) std::unique_ptr<std::vector<CXX_TYPE>>(raw);                     \
  }                                                                            \
  const std::vector<CXX_TYPE>                                                  \
      *cxxbridge03$unique_ptr$std$vector$##RUST_TYPE##$get(                    \
          const std::unique_ptr<std::vector<CXX_TYPE>> &ptr) noexcept {        \
    return ptr.get();                                                          \
  }                                                                            \
  std::vector<CXX_TYPE>                                                        \
      *cxxbridge03$unique_ptr$std$vector$##RUST_TYPE##$release(                \
          std::unique_ptr<std::vector<CXX_TYPE>> &ptr) noexcept {              \
    return ptr.release();                                                      \
  }                                                                            \
  void cxxbridge03$unique_ptr$std$vector$##RUST_TYPE##$drop(                   \
      std::unique_ptr<std::vector<CXX_TYPE>> *ptr) noexcept {                  \
    ptr->~unique_ptr();                                                        \
  }

#define RUST_VEC_EXTERNS(RUST_TYPE, CXX_TYPE)                                  \
  void cxxbridge03$rust_vec$##RUST_TYPE##$new(                                 \
      rust::Vec<CXX_TYPE> *ptr) noexcept;                                      \
  void cxxbridge03$rust_vec$##RUST_TYPE##$drop(                                \
      rust::Vec<CXX_TYPE> *ptr) noexcept;                                      \
  size_t cxxbridge03$rust_vec$##RUST_TYPE##$len(                               \
      const rust::Vec<CXX_TYPE> *ptr) noexcept;                                \
  const CXX_TYPE *cxxbridge03$rust_vec$##RUST_TYPE##$data(                     \
      const rust::Vec<CXX_TYPE> *ptr) noexcept;                                \
  size_t cxxbridge03$rust_vec$##RUST_TYPE##$stride() noexcept;

#define RUST_VEC_OPS(RUST_TYPE, CXX_TYPE)                                      \
  template <>                                                                  \
  Vec<CXX_TYPE>::Vec() noexcept {                                              \
    cxxbridge03$rust_vec$##RUST_TYPE##$new(this);                              \
  }                                                                            \
  template <>                                                                  \
  void Vec<CXX_TYPE>::drop() noexcept {                                        \
    return cxxbridge03$rust_vec$##RUST_TYPE##$drop(this);                      \
  }                                                                            \
  template <>                                                                  \
  size_t Vec<CXX_TYPE>::size() const noexcept {                                \
    return cxxbridge03$rust_vec$##RUST_TYPE##$len(this);                       \
  }                                                                            \
  template <>                                                                  \
  const CXX_TYPE *Vec<CXX_TYPE>::data() const noexcept {                       \
    return cxxbridge03$rust_vec$##RUST_TYPE##$data(this);                      \
  }                                                                            \
  template <>                                                                  \
  size_t Vec<CXX_TYPE>::stride() noexcept {                                    \
    return cxxbridge03$rust_vec$##RUST_TYPE##$stride();                        \
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
  MACRO(isize, rust::isize)

#define FOR_EACH_RUST_VEC(MACRO)                                               \
  FOR_EACH_NUMERIC(MACRO)                                                      \
  MACRO(bool, bool)

extern "C" {
FOR_EACH_STD_VECTOR(STD_VECTOR_OPS)
FOR_EACH_RUST_VEC(RUST_VEC_EXTERNS)
} // extern "C"

namespace rust {
inline namespace cxxbridge03 {
FOR_EACH_RUST_VEC(RUST_VEC_OPS)
} // namespace cxxbridge03
} // namespace rust
