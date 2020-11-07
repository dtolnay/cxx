#include "mmscenegraph/_cpp.h"
#include "mmscenegraph.h"
#include <cstdint>
#include <memory>
#include <new>
#include <string>
#include <type_traits>
#include <utility>

namespace rust {
inline namespace cxxbridge05 {
// #include "rust/cxx.h"

namespace {
template <typename T>
class impl;
} // namespace

#ifndef CXXBRIDGE05_RUST_STR
#define CXXBRIDGE05_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, size_t);
  Str(std::string &&) = delete;

  Str &operator=(const Str &) noexcept = default;

  explicit operator std::string() const;

  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

private:
  friend impl<Str>;
  const char *ptr;
  size_t len;
};

inline const char *Str::data() const noexcept { return this->ptr; }

inline size_t Str::size() const noexcept { return this->len; }

inline size_t Str::length() const noexcept { return this->len; }
#endif // CXXBRIDGE05_RUST_STR

#ifndef CXXBRIDGE05_RUST_BOX
#define CXXBRIDGE05_RUST_BOX
template <typename T>
class Box final {
public:
  using value_type = T;
  using const_pointer =
      typename std::add_pointer<typename std::add_const<T>::type>::type;
  using pointer = typename std::add_pointer<T>::type;

  Box(const Box &);
  Box(Box &&) noexcept;
  ~Box() noexcept;

  explicit Box(const T &);
  explicit Box(T &&);

  Box &operator=(const Box &);
  Box &operator=(Box &&) noexcept;

  const T *operator->() const noexcept;
  const T &operator*() const noexcept;
  T *operator->() noexcept;
  T &operator*() noexcept;

  template <typename... Fields>
  static Box in_place(Fields &&...);

  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

private:
  Box() noexcept;
  void uninit() noexcept;
  void drop() noexcept;
  T *ptr;
};

template <typename T>
Box<T>::Box(const Box &other) : Box(*other) {}

template <typename T>
Box<T>::Box(Box &&other) noexcept : ptr(other.ptr) {
  other.ptr = nullptr;
}

template <typename T>
Box<T>::Box(const T &val) {
  this->uninit();
  ::new (this->ptr) T(val);
}

template <typename T>
Box<T>::Box(T &&val) {
  this->uninit();
  ::new (this->ptr) T(std::move(val));
}

template <typename T>
Box<T>::~Box() noexcept {
  if (this->ptr) {
    this->drop();
  }
}

template <typename T>
Box<T> &Box<T>::operator=(const Box &other) {
  if (this != &other) {
    if (this->ptr) {
      **this = *other;
    } else {
      this->uninit();
      ::new (this->ptr) T(*other);
    }
  }
  return *this;
}

template <typename T>
Box<T> &Box<T>::operator=(Box &&other) noexcept {
  if (this->ptr) {
    this->drop();
  }
  this->ptr = other.ptr;
  other.ptr = nullptr;
  return *this;
}

template <typename T>
const T *Box<T>::operator->() const noexcept {
  return this->ptr;
}

template <typename T>
const T &Box<T>::operator*() const noexcept {
  return *this->ptr;
}

template <typename T>
T *Box<T>::operator->() noexcept {
  return this->ptr;
}

template <typename T>
T &Box<T>::operator*() noexcept {
  return *this->ptr;
}

template <typename T>
template <typename... Fields>
Box<T> Box<T>::in_place(Fields &&... fields) {
  Box box;
  box.uninit();
  ::new (box.ptr) T{std::forward<Fields>(fields)...};
  return box;
}

template <typename T>
Box<T> Box<T>::from_raw(T *raw) noexcept {
  Box box;
  box.ptr = raw;
  return box;
}

template <typename T>
T *Box<T>::into_raw() noexcept {
  T *raw = this->ptr;
  this->ptr = nullptr;
  return raw;
}

template <typename T>
Box<T>::Box() noexcept {}
#endif // CXXBRIDGE05_RUST_BOX

namespace {
namespace repr {
struct PtrLen final {
  const void *ptr;
  size_t len;
};
} // namespace repr

template <>
class impl<Str> final {
public:
  static Str new_unchecked(repr::PtrLen repr) noexcept {
    Str str;
    str.ptr = static_cast<const char *>(repr.ptr);
    str.len = repr.len;
    return str;
  }
};
} // namespace
} // namespace cxxbridge05
} // namespace rust

namespace mmscenegraph {
  struct SharedThing;
  using ThingC = ::mmscenegraph::ThingC;
  struct ThingR;
}

namespace mmscenegraph {
#ifndef CXXBRIDGE05_STRUCT_mmscenegraph$SharedThing
#define CXXBRIDGE05_STRUCT_mmscenegraph$SharedThing
struct SharedThing final {
  int32_t z;
  ::rust::Box<::mmscenegraph::ThingR> y;
  ::std::unique_ptr<::mmscenegraph::ThingC> x;
};
#endif // CXXBRIDGE05_STRUCT_mmscenegraph$SharedThing

extern "C" {
__declspec(dllexport) ::mmscenegraph::ThingC *mmscenegraph$cxxbridge05$make_demo(::rust::repr::PtrLen appname) noexcept {
  ::std::unique_ptr<::mmscenegraph::ThingC> (*make_demo$)(::rust::Str) = ::mmscenegraph::make_demo;
  return make_demo$(::rust::impl<::rust::Str>::new_unchecked(appname)).release();
}

__declspec(dllexport) const ::std::string *mmscenegraph$cxxbridge05$get_name(const ::mmscenegraph::ThingC &thing) noexcept {
  const ::std::string &(*get_name$)(const ::mmscenegraph::ThingC &) = ::mmscenegraph::get_name;
  return &get_name$(thing);
}

__declspec(dllexport) void mmscenegraph$cxxbridge05$do_thing(::mmscenegraph::SharedThing *state) noexcept {
  void (*do_thing$)(::mmscenegraph::SharedThing) = ::mmscenegraph::do_thing;
  do_thing$(::std::move(*state));
}

void mmscenegraph$cxxbridge05$print_r(const ::mmscenegraph::ThingR &r) noexcept;
} // extern "C"

void print_r(const ::mmscenegraph::ThingR &r) noexcept {
  mmscenegraph$cxxbridge05$print_r(r);
}
} // namespace mmscenegraph

extern "C" {
#ifndef CXXBRIDGE05_RUST_BOX_mmscenegraph$ThingR
#define CXXBRIDGE05_RUST_BOX_mmscenegraph$ThingR
void cxxbridge05$box$mmscenegraph$ThingR$uninit(::rust::Box<::mmscenegraph::ThingR> *ptr) noexcept;
void cxxbridge05$box$mmscenegraph$ThingR$drop(::rust::Box<::mmscenegraph::ThingR> *ptr) noexcept;
#endif // CXXBRIDGE05_RUST_BOX_mmscenegraph$ThingR

#ifndef CXXBRIDGE05_UNIQUE_PTR_mmscenegraph$ThingC
#define CXXBRIDGE05_UNIQUE_PTR_mmscenegraph$ThingC
static_assert(sizeof(::std::unique_ptr<::mmscenegraph::ThingC>) == sizeof(void *), "");
static_assert(alignof(::std::unique_ptr<::mmscenegraph::ThingC>) == alignof(void *), "");
void cxxbridge05$unique_ptr$mmscenegraph$ThingC$null(::std::unique_ptr<::mmscenegraph::ThingC> *ptr) noexcept {
  new (ptr) ::std::unique_ptr<::mmscenegraph::ThingC>();
}
void cxxbridge05$unique_ptr$mmscenegraph$ThingC$raw(::std::unique_ptr<::mmscenegraph::ThingC> *ptr, ::mmscenegraph::ThingC *raw) noexcept {
  new (ptr) ::std::unique_ptr<::mmscenegraph::ThingC>(raw);
}
const ::mmscenegraph::ThingC *cxxbridge05$unique_ptr$mmscenegraph$ThingC$get(const ::std::unique_ptr<::mmscenegraph::ThingC>& ptr) noexcept {
  return ptr.get();
}
::mmscenegraph::ThingC *cxxbridge05$unique_ptr$mmscenegraph$ThingC$release(::std::unique_ptr<::mmscenegraph::ThingC>& ptr) noexcept {
  return ptr.release();
}
void cxxbridge05$unique_ptr$mmscenegraph$ThingC$drop(::std::unique_ptr<::mmscenegraph::ThingC> *ptr) noexcept {
  ptr->~unique_ptr();
}
#endif // CXXBRIDGE05_UNIQUE_PTR_mmscenegraph$ThingC
} // extern "C"

namespace rust {
inline namespace cxxbridge05 {
template <>
void Box<::mmscenegraph::ThingR>::uninit() noexcept {
  cxxbridge05$box$mmscenegraph$ThingR$uninit(this);
}
template <>
void Box<::mmscenegraph::ThingR>::drop() noexcept {
  cxxbridge05$box$mmscenegraph$ThingR$drop(this);
}
} // namespace cxxbridge05
} // namespace rust
