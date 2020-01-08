#include "../include/cxxbridge.h"
#include <cstring>
#include <memory>
#include <stdexcept>

namespace cxxbridge = cxxbridge01;

extern "C" {
const char *cxxbridge01$cxx_string$data(const std::string &s) noexcept {
  return s.data();
}

size_t cxxbridge01$cxx_string$length(const std::string &s) noexcept {
  return s.length();
}

// RustString
void cxxbridge01$rust_string$new(cxxbridge::RustString *self) noexcept;
void cxxbridge01$rust_string$clone(cxxbridge::RustString *self,
                                   const cxxbridge::RustString &other) noexcept;
bool cxxbridge01$rust_string$from(cxxbridge::RustString *self, const char *ptr,
                                  size_t len) noexcept;
void cxxbridge01$rust_string$drop(cxxbridge::RustString *self) noexcept;
const char *
cxxbridge01$rust_string$ptr(const cxxbridge::RustString *self) noexcept;
size_t cxxbridge01$rust_string$len(const cxxbridge::RustString *self) noexcept;

// RustStr
bool cxxbridge01$rust_str$valid(const char *ptr, size_t len) noexcept;
} // extern "C"

namespace cxxbridge01 {

RustString::RustString() noexcept { cxxbridge01$rust_string$new(this); }

RustString::RustString(const RustString &other) noexcept {
  cxxbridge01$rust_string$clone(this, other);
}

RustString::RustString(RustString &&other) noexcept {
  this->repr = other.repr;
  cxxbridge01$rust_string$new(&other);
}

RustString::RustString(const char *s) {
  auto len = strlen(s);
  if (!cxxbridge01$rust_string$from(this, s, len)) {
    throw std::invalid_argument("data for RustString is not utf-8");
  }
}

RustString::RustString(const std::string &s) {
  auto ptr = s.data();
  auto len = s.length();
  if (!cxxbridge01$rust_string$from(this, ptr, len)) {
    throw std::invalid_argument("data for RustString is not utf-8");
  }
}

RustString::~RustString() noexcept { cxxbridge01$rust_string$drop(this); }

RustString::operator std::string() const {
  return std::string(this->data(), this->size());
}

RustString &RustString::operator=(const RustString &other) noexcept {
  if (this != &other) {
    cxxbridge01$rust_string$drop(this);
    cxxbridge01$rust_string$clone(this, other);
  }
  return *this;
}

RustString &RustString::operator=(RustString &&other) noexcept {
  if (this != &other) {
    cxxbridge01$rust_string$drop(this);
    this->repr = other.repr;
    cxxbridge01$rust_string$new(&other);
  }
  return *this;
}

const char *RustString::data() const noexcept {
  return cxxbridge01$rust_string$ptr(this);
}

size_t RustString::size() const noexcept {
  return cxxbridge01$rust_string$len(this);
}

size_t RustString::length() const noexcept {
  return cxxbridge01$rust_string$len(this);
}

std::ostream &operator<<(std::ostream &os, const RustString &s) {
  os.write(s.data(), s.size());
  return os;
}

RustStr::RustStr() noexcept
    : repr(Repr{reinterpret_cast<const char *>(this), 0}) {}

RustStr::RustStr(const char *s) : repr(Repr{s, strlen(s)}) {
  if (!cxxbridge01$rust_str$valid(this->repr.ptr, this->repr.len)) {
    throw std::invalid_argument("data for RustStr is not utf-8");
  }
}

RustStr::RustStr(const std::string &s) : repr(Repr{s.data(), s.length()}) {
  if (!cxxbridge01$rust_str$valid(this->repr.ptr, this->repr.len)) {
    throw std::invalid_argument("data for RustStr is not utf-8");
  }
}

RustStr::RustStr(const RustStr &) noexcept = default;

RustStr &RustStr::operator=(RustStr other) noexcept {
  this->repr = other.repr;
  return *this;
}

RustStr::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *RustStr::data() const noexcept { return this->repr.ptr; }

size_t RustStr::size() const noexcept { return this->repr.len; }

size_t RustStr::length() const noexcept { return this->repr.len; }

RustStr::RustStr(Repr repr_) noexcept : repr(repr_) {}

RustStr::operator Repr() noexcept { return this->repr; }

std::ostream &operator<<(std::ostream &os, const RustStr &s) {
  os.write(s.data(), s.size());
  return os;
}

} // namespace cxxbridge01

extern "C" {
void cxxbridge01$unique_ptr$std$string$null(
    std::unique_ptr<std::string> *ptr) noexcept {
  new (ptr) std::unique_ptr<std::string>();
}
void cxxbridge01$unique_ptr$std$string$new(std::unique_ptr<std::string> *ptr,
                                           std::string *value) noexcept {
  new (ptr) std::unique_ptr<std::string>(new std::string(std::move(*value)));
}
void cxxbridge01$unique_ptr$std$string$raw(std::unique_ptr<std::string> *ptr,
                                           std::string *raw) noexcept {
  new (ptr) std::unique_ptr<std::string>(raw);
}
const std::string *cxxbridge01$unique_ptr$std$string$get(
    const std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.get();
}
std::string *cxxbridge01$unique_ptr$std$string$release(
    std::unique_ptr<std::string> &ptr) noexcept {
  return ptr.release();
}
void cxxbridge01$unique_ptr$std$string$drop(
    std::unique_ptr<std::string> *ptr) noexcept {
  ptr->~unique_ptr();
}
} // extern "C"
