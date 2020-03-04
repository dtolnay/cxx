#include "../include/cxxbridge.h"
#include <cstring>
#include <iostream>
#include <memory>
#include <stdexcept>

extern "C" {
const char *cxxbridge01$cxx_string$data(const std::string &s) noexcept {
  return s.data();
}

size_t cxxbridge01$cxx_string$length(const std::string &s) noexcept {
  return s.length();
}

// rust::String
void cxxbridge01$string$new(rust::String *self) noexcept;
void cxxbridge01$string$clone(rust::String *self,
                              const rust::String &other) noexcept;
bool cxxbridge01$string$from(rust::String *self, const char *ptr,
                             size_t len) noexcept;
void cxxbridge01$string$drop(rust::String *self) noexcept;
const char *cxxbridge01$string$ptr(const rust::String *self) noexcept;
size_t cxxbridge01$string$len(const rust::String *self) noexcept;

// rust::Str
bool cxxbridge01$str$valid(const char *ptr, size_t len) noexcept;
} // extern "C"

namespace rust {
inline namespace cxxbridge01 {

String::String() noexcept { cxxbridge01$string$new(this); }

String::String(const String &other) noexcept {
  cxxbridge01$string$clone(this, other);
}

String::String(String &&other) noexcept {
  this->repr = other.repr;
  cxxbridge01$string$new(&other);
}

String::~String() noexcept { cxxbridge01$string$drop(this); }

String::String(const std::string &s) {
  auto ptr = s.data();
  auto len = s.length();
  if (!cxxbridge01$string$from(this, ptr, len)) {
    throw std::invalid_argument("data for rust::String is not utf-8");
  }
}

String::String(const char *s) {
  auto len = strlen(s);
  if (!cxxbridge01$string$from(this, s, len)) {
    throw std::invalid_argument("data for rust::String is not utf-8");
  }
}

String &String::operator=(const String &other) noexcept {
  if (this != &other) {
    cxxbridge01$string$drop(this);
    cxxbridge01$string$clone(this, other);
  }
  return *this;
}

String &String::operator=(String &&other) noexcept {
  if (this != &other) {
    cxxbridge01$string$drop(this);
    this->repr = other.repr;
    cxxbridge01$string$new(&other);
  }
  return *this;
}

String::operator std::string() const {
  return std::string(this->data(), this->size());
}

const char *String::data() const noexcept {
  return cxxbridge01$string$ptr(this);
}

size_t String::size() const noexcept { return cxxbridge01$string$len(this); }

size_t String::length() const noexcept { return cxxbridge01$string$len(this); }

String::String(unsafe_bitcopy_t, const String &bits) noexcept
    : repr(bits.repr) {}

std::ostream &operator<<(std::ostream &os, const String &s) {
  os.write(s.data(), s.size());
  return os;
}

Str::Str() noexcept : repr(Repr{reinterpret_cast<const char *>(this), 0}) {}

Str::Str(const Str &) noexcept = default;

Str::Str(const std::string &s) : repr(Repr{s.data(), s.length()}) {
  if (!cxxbridge01$str$valid(this->repr.ptr, this->repr.len)) {
    throw std::invalid_argument("data for rust::Str is not utf-8");
  }
}

Str::Str(const char *s) : repr(Repr{s, strlen(s)}) {
  if (!cxxbridge01$str$valid(this->repr.ptr, this->repr.len)) {
    throw std::invalid_argument("data for rust::Str is not utf-8");
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

} // namespace cxxbridge01
} // namespace rust

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
