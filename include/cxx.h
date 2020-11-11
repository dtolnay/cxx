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
#include <basetsd.h>
#endif

namespace rust {
inline namespace cxxbridge05 {

struct unsafe_bitcopy_t;

namespace {
template <typename T>
class impl;
}

#ifndef CXXBRIDGE05_RUST_STRING
#define CXXBRIDGE05_RUST_STRING
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
#endif // CXXBRIDGE05_RUST_STRING

#ifndef CXXBRIDGE05_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, size_t);
  Str(std::string &&) = delete;

  Str &operator=(const Str &) noexcept = default;

  explicit operator std::string() const;

  // Note: no null terminator.
  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

private:
  friend impl<Str>;
  // Not necessarily ABI compatible with &str. Codegen will translate to
  // cxx::rust_str::RustStr which matches this layout.
  const char *ptr;
  size_t len;
};
#endif // CXXBRIDGE05_RUST_STR

#ifndef CXXBRIDGE05_RUST_SLICE
template <typename T>
class Slice final {
public:
  Slice() noexcept;
  Slice(const T *, size_t count) noexcept;

  Slice &operator=(const Slice<T> &) noexcept = default;

  const T *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

private:
  friend impl<Slice>;
  // Not necessarily ABI compatible with &[T]. Codegen will translate to
  // cxx::rust_sliceu8::RustSliceU8 which matches this layout.
  const T *ptr;
  size_t len;
};
#endif // CXXBRIDGE05_RUST_SLICE

#ifndef CXXBRIDGE05_RUST_BOX
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
#endif // CXXBRIDGE05_RUST_BOX

#ifndef CXXBRIDGE05_RUST_VEC
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
  T *data() noexcept;

  const T &operator[](size_t n) const noexcept;
  const T &at(size_t n) const;

  const T &front() const;
  const T &back() const;

  void reserve(size_t new_cap);
  void push_back(const T &value);
  void push_back(T &&value);
  template <class... Args>
  void emplace_back(Args &&... args);

  class const_iterator final {
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
  void reserve_total(size_t cap) noexcept;
  void set_len(size_t len) noexcept;
  void drop() noexcept;

  // Size and alignment statically verified by rust_vec.rs.
  std::array<uintptr_t, 3> repr;
};
#endif // CXXBRIDGE05_RUST_VEC

#ifndef CXXBRIDGE05_RUST_FN
template <typename Signature, bool Throws = false>
class Fn;

template <typename Ret, typename... Args, bool Throws>
class Fn<Ret(Args...), Throws> final {
public:
  Ret operator()(Args... args) const noexcept(!Throws);
  Fn operator*() const noexcept;

private:
  Ret (*trampoline)(Args..., void *fn) noexcept(!Throws);
  void *fn;
};

template <typename Signature>
using TryFn = Fn<Signature, true>;
#endif // CXXBRIDGE05_RUST_FN

#ifndef CXXBRIDGE05_RUST_ERROR
#define CXXBRIDGE05_RUST_ERROR
class Error final : public std::exception {
public:
  Error(const Error &);
  Error(Error &&) noexcept;
  ~Error() noexcept;

  Error &operator=(const Error &);
  Error &operator=(Error &&) noexcept;

  const char *what() const noexcept override;

private:
  Error() noexcept = default;
  friend impl<Error>;
  const char *msg;
  size_t len;
};
#endif // CXXBRIDGE05_RUST_ERROR

#ifndef CXXBRIDGE05_RUST_ISIZE
#define CXXBRIDGE05_RUST_ISIZE
#if defined(_WIN32)
using isize = SSIZE_T;
#else
using isize = ssize_t;
#endif
#endif // CXXBRIDGE05_RUST_ISIZE

std::ostream &operator<<(std::ostream &, const String &);
std::ostream &operator<<(std::ostream &, const Str &);

// IsRelocatable<T> is used in assertions that a C++ type passed by value
// between Rust and C++ is soundly relocatable by Rust.
//
// There may be legitimate reasons to opt out of the check for support of types
// that the programmer knows are soundly Rust-movable despite not being
// recognized as such by the C++ type system due to a move constructor or
// destructor. To opt out of the relocatability check, do either of the
// following things in any header used by `include!` in the bridge.
//
//      --- if you define the type:
//      struct MyType {
//        ...
//    +   using IsRelocatable = std::true_type;
//      };
//
//      --- otherwise:
//    + template <>
//    + struct rust::IsRelocatable<MyType> : std::true_type {};
template <typename T>
struct IsRelocatable;

// Snake case aliases for use in code that uses this style for type names.
using string = String;
using str = Str;
template <class T>
using slice = Slice<T>;
template <class T>
using box = Box<T>;
template <class T>
using vec = Vec<T>;
using error = Error;
template <typename Signature, bool Throws = false>
using fn = Fn<Signature, Throws>;
template <typename Signature>
using try_fn = TryFn<Signature>;
template <typename T>
using is_relocatable = IsRelocatable<T>;



////////////////////////////////////////////////////////////////////////////////
/// end public API, begin implementation details

#ifndef CXXBRIDGE05_PANIC
#define CXXBRIDGE05_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE05_PANIC

#ifndef CXXBRIDGE05_RUST_FN
#define CXXBRIDGE05_RUST_FN
template <typename Ret, typename... Args, bool Throws>
Ret Fn<Ret(Args...), Throws>::operator()(Args... args) const noexcept(!Throws) {
  return (*this->trampoline)(std::move(args)..., this->fn);
}

template <typename Ret, typename... Args, bool Throws>
Fn<Ret(Args...), Throws> Fn<Ret(Args...), Throws>::operator*() const noexcept {
  return *this;
}
#endif // CXXBRIDGE05_RUST_FN

#ifndef CXXBRIDGE05_RUST_BITCOPY
#define CXXBRIDGE05_RUST_BITCOPY
struct unsafe_bitcopy_t final {
  explicit unsafe_bitcopy_t() = default;
};

constexpr unsafe_bitcopy_t unsafe_bitcopy{};
#endif // CXXBRIDGE05_RUST_BITCOPY

#ifndef CXXBRIDGE05_RUST_STR
#define CXXBRIDGE05_RUST_STR
inline const char *Str::data() const noexcept { return this->ptr; }

inline size_t Str::size() const noexcept { return this->len; }

inline size_t Str::length() const noexcept { return this->len; }
#endif // CXXBRIDGE05_RUST_STR

#ifndef CXXBRIDGE05_RUST_SLICE
#define CXXBRIDGE05_RUST_SLICE
template <typename T>
Slice<T>::Slice() noexcept : ptr(reinterpret_cast<const T *>(this)), len(0) {}

template <typename T>
Slice<T>::Slice(const T *s, size_t count) noexcept : ptr(s), len(count) {}

template <typename T>
const T *Slice<T>::data() const noexcept {
  return this->ptr;
}

template <typename T>
size_t Slice<T>::size() const noexcept {
  return this->len;
}

template <typename T>
size_t Slice<T>::length() const noexcept {
  return this->len;
}
#endif // CXXBRIDGE05_RUST_SLICE

#ifndef CXXBRIDGE05_RUST_BOX
#define CXXBRIDGE05_RUST_BOX
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

#ifndef CXXBRIDGE05_RUST_VEC
#define CXXBRIDGE05_RUST_VEC
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
T *Vec<T>::data() noexcept {
  return const_cast<T *>(const_cast<const Vec<T> *>(this)->data());
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
void Vec<T>::reserve(size_t new_cap) {
  this->reserve_total(new_cap);
}

template <typename T>
void Vec<T>::push_back(const T &value) {
  this->emplace_back(value);
}

template <typename T>
void Vec<T>::push_back(T &&value) {
  this->emplace_back(std::move(value));
}

template <typename T>
template <typename... Args>
void Vec<T>::emplace_back(Args &&... args) {
  auto size = this->size();
  this->reserve_total(size + 1);
  ::new (reinterpret_cast<T *>(reinterpret_cast<char *>(this->data()) +
                               size * this->stride()))
      T(std::forward<Args>(args)...);
  this->set_len(size + 1);
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
#endif // CXXBRIDGE05_RUST_VEC

#ifndef CXXBRIDGE05_RELOCATABLE
#define CXXBRIDGE05_RELOCATABLE
namespace detail {
template <typename... Ts>
struct make_void {
  using type = void;
};

template <typename... Ts>
using void_t = typename make_void<Ts...>::type;

template <typename Void, template <typename...> class, typename...>
struct detect : std::false_type {};
template <template <typename...> class T, typename... A>
struct detect<void_t<T<A...>>, T, A...> : std::true_type {};

template <template <typename...> class T, typename... A>
using is_detected = detect<void, T, A...>;

template <typename T>
using detect_IsRelocatable = typename T::IsRelocatable;

template <typename T>
struct get_IsRelocatable
    : std::is_same<typename T::IsRelocatable, std::true_type> {};
} // namespace detail

template <typename T>
struct IsRelocatable
    : std::conditional<
          detail::is_detected<detail::detect_IsRelocatable, T>::value,
          detail::get_IsRelocatable<T>,
          std::integral_constant<
              bool, std::is_trivially_move_constructible<T>::value &&
                        std::is_trivially_destructible<T>::value>>::type {};
#endif // CXXBRIDGE05_RELOCATABLE

} // namespace cxxbridge05
} // namespace rust
