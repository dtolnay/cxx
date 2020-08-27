#pragma once
#include <array>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <iosfwd>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>
#if defined(_WIN32)
#include <BaseTsd.h>
#endif

namespace rust {
inline namespace cxxbridge03 {

struct unsafe_bitcopy_t;

#ifndef CXXBRIDGE03_RUST_STRING
#define CXXBRIDGE03_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, size_t);

  String &operator=(const String &) noexcept;
  String &operator=(String &&) noexcept;

  explicit operator std::string() const;

  // Note: no null terminator.
  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Internal API only intended for the cxxbridge code generator.
  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  // Size and alignment statically verified by rust_string.rs.
  std::array<uintptr_t, 3> repr;
};
#endif // CXXBRIDGE03_RUST_STRING

#ifndef CXXBRIDGE03_RUST_STR
#define CXXBRIDGE03_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const Str &) noexcept;

  Str(const std::string &);
  Str(const char *);
  Str(const char *, size_t);
  Str(std::string &&) = delete;

  Str &operator=(Str) noexcept;

  explicit operator std::string() const;

  // Note: no null terminator.
  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Repr is PRIVATE; must not be used other than by our generated code.
  //
  // Not necessarily ABI compatible with &str. Codegen will translate to
  // cxx::rust_str::RustStr which matches this layout.
  struct Repr {
    const char *ptr;
    size_t len;
  };
  Str(Repr) noexcept;
  explicit operator Repr() noexcept;

private:
  Repr repr;
};
#endif // CXXBRIDGE03_RUST_STR

#ifndef CXXBRIDGE03_RUST_SLICE
template <typename T>
class Slice final {
public:
  Slice() noexcept;
  Slice(const Slice<T> &) noexcept;
  Slice(const T *, size_t count) noexcept;

  Slice &operator=(Slice<T>) noexcept;

  const T *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Repr is PRIVATE; must not be used other than by our generated code.
  //
  // At present this class is only used for &[u8] slices.
  // Not necessarily ABI compatible with &[u8]. Codegen will translate to
  // cxx::rust_sliceu8::RustSliceU8 which matches this layout.
  struct Repr {
    const T *ptr;
    size_t len;
  };
  Slice(Repr) noexcept;
  explicit operator Repr() noexcept;

private:
  Repr repr;
};
#endif // CXXBRIDGE03_RUST_SLICE

#ifndef CXXBRIDGE03_RUST_BOX
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

  // Important: requires that `raw` came from an into_raw call. Do not pass a
  // pointer from `new` or any other source.
  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

private:
  Box() noexcept;
  void uninit() noexcept;
  void drop() noexcept;
  T *ptr;
};
#endif // CXXBRIDGE03_RUST_BOX

#ifndef CXXBRIDGE03_RUST_VEC
template <typename T>
class Vec final {
public:
  using value_type = T;

  Vec() noexcept;
  Vec(Vec &&) noexcept;
  ~Vec() noexcept;

  Vec &operator=(Vec &&) noexcept;

  size_t size() const noexcept;
  bool empty() const noexcept;
  const T *data() const noexcept;

  const T &operator[](size_t n) const noexcept;
  const T &at(size_t n) const;

  const T &front() const;
  const T &back() const;

  class const_iterator {
  public:
    using difference_type = ptrdiff_t;
    using value_type = typename std::add_const<T>::type;
    using pointer =
        typename std::add_pointer<typename std::add_const<T>::type>::type;
    using reference = typename std::add_lvalue_reference<
        typename std::add_const<T>::type>::type;
    using iterator_category = std::forward_iterator_tag;

    const T &operator*() const noexcept;
    const T *operator->() const noexcept;
    const_iterator &operator++() noexcept;
    const_iterator operator++(int) noexcept;
    bool operator==(const const_iterator &) const noexcept;
    bool operator!=(const const_iterator &) const noexcept;

  private:
    friend class Vec;
    const void *pos;
    size_t stride;
  };

  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;

  // Internal API only intended for the cxxbridge code generator.
  Vec(unsafe_bitcopy_t, const Vec &) noexcept;

private:
  static size_t stride() noexcept;
  void drop() noexcept;

  // Size and alignment statically verified by rust_vec.rs.
  std::array<uintptr_t, 3> repr;
};
#endif // CXXBRIDGE03_RUST_VEC

#ifndef CXXBRIDGE03_RUST_FN
#define CXXBRIDGE03_RUST_FN
template <typename Signature, bool Throws = false>
class Fn;

template <typename Ret, typename... Args, bool Throws>
class Fn<Ret(Args...), Throws> {
public:
  Ret operator()(Args... args) const noexcept(!Throws);
  Fn operator*() const noexcept;

private:
  Ret (*trampoline)(Args..., void *fn) noexcept(!Throws);
  void *fn;
};

template <typename Signature>
using TryFn = Fn<Signature, true>;
#endif // CXXBRIDGE03_RUST_FN

#ifndef CXXBRIDGE03_RUST_ERROR
#define CXXBRIDGE03_RUST_ERROR
class Error final : std::exception {
public:
  Error(const Error &);
  Error(Error &&) noexcept;
  Error(Str::Repr) noexcept;
  ~Error() noexcept;
  const char *what() const noexcept override;

private:
  Str::Repr msg;
};
#endif // CXXBRIDGE03_RUST_ERROR

#ifndef CXXBRIDGE03_RUST_ISIZE
#define CXXBRIDGE03_RUST_ISIZE
#if defined(_WIN32)
using isize = SSIZE_T;
#else
using isize = ssize_t;
#endif
#endif // CXXBRIDGE03_RUST_ISIZE

std::ostream &operator<<(std::ostream &, const String &);
std::ostream &operator<<(std::ostream &, const Str &);

// Snake case aliases for use in code that uses this style for type names.
using string = String;
using str = Str;
template <class T>
using box = Box<T>;
using error = Error;
template <typename Signature, bool Throws = false>
using fn = Fn<Signature, Throws>;
template <typename Signature>
using try_fn = TryFn<Signature>;



////////////////////////////////////////////////////////////////////////////////
/// end public API, begin implementation details

template <typename Exception>
void panic [[noreturn]] (const char *msg);

template <typename Ret, typename... Args, bool Throws>
Ret Fn<Ret(Args...), Throws>::operator()(Args... args) const noexcept(!Throws) {
  return (*this->trampoline)(std::move(args)..., this->fn);
}

template <typename Ret, typename... Args, bool Throws>
Fn<Ret(Args...), Throws> Fn<Ret(Args...), Throws>::operator*() const noexcept {
  return *this;
}

#ifndef CXXBRIDGE03_RUST_BITCOPY
#define CXXBRIDGE03_RUST_BITCOPY
struct unsafe_bitcopy_t {
  explicit unsafe_bitcopy_t() = default;
};

constexpr unsafe_bitcopy_t unsafe_bitcopy{};
#endif // CXXBRIDGE03_RUST_BITCOPY

#ifndef CXXBRIDGE03_RUST_SLICE
#define CXXBRIDGE03_RUST_SLICE
template <typename T>
Slice<T>::Slice() noexcept : repr(Repr{reinterpret_cast<const T *>(this), 0}) {}

template <typename T>
Slice<T>::Slice(const Slice<T> &) noexcept = default;

template <typename T>
Slice<T>::Slice(const T *s, size_t count) noexcept : repr(Repr{s, count}) {}

template <typename T>
Slice<T> &Slice<T>::operator=(Slice<T> other) noexcept {
  this->repr = other.repr;
  return *this;
}

template <typename T>
const T *Slice<T>::data() const noexcept {
  return this->repr.ptr;
}

template <typename T>
size_t Slice<T>::size() const noexcept {
  return this->repr.len;
}

template <typename T>
size_t Slice<T>::length() const noexcept {
  return this->repr.len;
}

template <typename T>
Slice<T>::Slice(Repr repr_) noexcept : repr(repr_) {}

template <typename T>
Slice<T>::operator Repr() noexcept {
  return this->repr;
}
#endif // CXXBRIDGE03_RUST_SLICE

#ifndef CXXBRIDGE03_RUST_BOX
#define CXXBRIDGE03_RUST_BOX
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
#endif // CXXBRIDGE03_RUST_BOX

#ifndef CXXBRIDGE03_RUST_VEC
#define CXXBRIDGE03_RUST_VEC
template <typename T>
Vec<T>::Vec(Vec &&other) noexcept {
  this->repr = other.repr;
  new (&other) Vec();
}

template <typename T>
Vec<T>::~Vec() noexcept {
  this->drop();
}

template <typename T>
Vec<T> &Vec<T>::operator=(Vec &&other) noexcept {
  if (this != &other) {
    this->drop();
    this->repr = other.repr;
    new (&other) Vec();
  }
  return *this;
}

template <typename T>
bool Vec<T>::empty() const noexcept {
  return size() == 0;
}

template <typename T>
const T &Vec<T>::operator[](size_t n) const noexcept {
  auto data = reinterpret_cast<const char *>(this->data());
  return *reinterpret_cast<const T *>(data + n * this->stride());
}

template <typename T>
const T &Vec<T>::at(size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
const T &Vec<T>::front() const {
  return (*this)[0];
}

template <typename T>
const T &Vec<T>::back() const {
  return (*this)[this->size() - 1];
}

template <typename T>
const T &Vec<T>::const_iterator::operator*() const noexcept {
  return *static_cast<const T *>(this->pos);
}

template <typename T>
const T *Vec<T>::const_iterator::operator->() const noexcept {
  return static_cast<const T *>(this->pos);
}

template <typename T>
typename Vec<T>::const_iterator &Vec<T>::const_iterator::operator++() noexcept {
  this->pos = static_cast<const uint8_t *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Vec<T>::const_iterator
Vec<T>::const_iterator::operator++(int) noexcept {
  auto ret = const_iterator(*this);
  this->pos = static_cast<const uint8_t *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
bool Vec<T>::const_iterator::operator==(
    const const_iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Vec<T>::const_iterator::operator!=(
    const const_iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::begin() const noexcept {
  const_iterator it;
  it.pos = this->data();
  it.stride = this->stride();
  return it;
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::end() const noexcept {
  const_iterator it = this->begin();
  it.pos = static_cast<const uint8_t *>(it.pos) + it.stride * this->size();
  return it;
}

// Internal API only intended for the cxxbridge code generator.
template <typename T>
Vec<T>::Vec(unsafe_bitcopy_t, const Vec &bits) noexcept : repr(bits.repr) {}
#endif // CXXBRIDGE03_RUST_VEC

} // namespace cxxbridge03
} // namespace rust
