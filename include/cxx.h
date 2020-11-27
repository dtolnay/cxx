#pragma once
#include <array>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <iosfwd>
#include <iterator>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
#include <vector>
#if defined(_WIN32)
#include <basetsd.h>
#endif

#ifdef CXXBRIDGE_HAS_STRING_VIEW
#error "string_view gets auto-detected"
#endif

// clang-format off
// detect if string_view supported (C++17 required)
#ifdef __has_include
  #if __has_include(<version>)
    // gcc >= 9, clang >= 9, msvc >= 19.22
    #include <version>
    #if __cpp_lib_string_view >= 201603L
      #define CXXBRIDGE_HAS_STRING_VIEW
    #endif
  #else
    // no <version> include
    #if defined(__cpp_lib_string_view) && __cpp_lib_string_view >= 201603L
      // gcc: 8, clang: 5 - 8, msvc: 19.15 - 19.21
      #define CXXBRIDGE_HAS_STRING_VIEW
    #else
      // only include if compiled with c++17
      #if (__GNUC__ >= 7 && __cplusplus >= 201703L) || (_MSVC_LANG >= 201703L)
        // gcc: 7, msvc: 19.14
        #define CXXBRIDGE_HAS_STRING_VIEW
      #endif
    #endif
  #endif
#else
  // no __has_include
  #if _MSC_VER >= 1910L && _MSVC_LANG > 201402L
    // msvc: 19.10 requires c++latest (!)
    #define CPPBRIDGE_HAS_STRING_VIEW
  #endif
#endif
// clang-format on

#ifdef CXXBRIDGE_HAS_STRING_VIEW
#include <string_view>
#endif

namespace rust {
inline namespace cxxbridge1 {

struct unsafe_bitcopy_t;

namespace {
template <typename T>
class impl;
}

#ifndef CXXBRIDGE1_RUST_STRING
#define CXXBRIDGE1_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

#ifdef CXXBRIDGE_HAS_STRING_VIEW
  String(std::string_view sv) : String(sv.data(), sv.length()) {}
#endif
  String(const std::string &);
  String(const char *);
  String(const char *, size_t);

  String &operator=(const String &) noexcept;
  String &operator=(String &&) noexcept;

  explicit operator std::string() const;
#ifdef CXXBRIDGE_HAS_STRING_VIEW
  explicit operator std::string_view() const noexcept {
    return {this->data(), this->size()};
  }
#endif

  // Note: no null terminator.
  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  // Internal API only intended for the cxxbridge code generator.
  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  // Size and alignment statically verified by rust_string.rs.
  std::array<uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

#ifndef CXXBRIDGE1_RUST_STR
class Str final {
public:
  Str() noexcept;
#ifdef CXXBRIDGE_HAS_STRING_VIEW
  Str(std::string_view sv) : Str(sv.data(), sv.length()) {}
#endif
  Str(const std::string &);
  Str(const char *);
  Str(const char *, size_t);

  Str &operator=(const Str &) noexcept = default;

  explicit operator std::string() const;
#ifdef CXXBRIDGE_HAS_STRING_VIEW
  explicit operator std::string_view() const noexcept {
    return {this->data(), this->size()};
  }
#endif

  // Note: no null terminator.
  const char *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

private:
  friend impl<Str>;
  // Not necessarily ABI compatible with &str. Codegen will translate to
  // cxx::rust_str::RustStr which matches this layout.
  const char *ptr;
  size_t len;
};
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
namespace detail {
template <bool>
struct copy_assignable_if {};

template <>
struct copy_assignable_if<false> {
  copy_assignable_if() noexcept = default;
  copy_assignable_if(const copy_assignable_if &) noexcept = default;
  copy_assignable_if &operator=(const copy_assignable_if &) noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) noexcept = default;
};
} // namespace detail

template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  Slice() noexcept;
  Slice(T *, size_t count) noexcept;

  Slice &operator=(const Slice<T> &) noexcept = default;
  Slice &operator=(Slice<T> &&) noexcept = default;

  T *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

private:
  friend impl<Slice>;
  // Not necessarily ABI compatible with &[T]. Codegen will translate to
  // cxx::rust_slice::RustSlice which matches this layout.
  T *ptr;
  size_t len;
};

template <typename T>
class Slice<T>::iterator final {
public:
  using difference_type = ptrdiff_t;
  using value_type = T;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;
  using iterator_category = std::forward_iterator_tag;

  T &operator*() const noexcept;
  T *operator->() const noexcept;
  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;

private:
  friend class Slice;
  T *pos;
};
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
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
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_VEC
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
  template <typename... Args>
  void emplace_back(Args &&... args);

  class iterator;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = typename Vec<const T>::iterator;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

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

template <typename T>
class Vec<T>::iterator final {
public:
  using difference_type = ptrdiff_t;
  using value_type = T;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;
  using iterator_category = std::forward_iterator_tag;

  T &operator*() const noexcept;
  T *operator->() const noexcept;
  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;

private:
  friend class Vec;
  friend class Vec<typename std::remove_const<T>::type>;
  void *pos;
  size_t stride;
};
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_RUST_FN
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
#endif // CXXBRIDGE1_RUST_FN

#ifndef CXXBRIDGE1_RUST_ERROR
#define CXXBRIDGE1_RUST_ERROR
class Error final : public std::exception {
public:
  Error(const Error &);
  Error(Error &&) noexcept;
  ~Error() noexcept override;

  Error &operator=(const Error &);
  Error &operator=(Error &&) noexcept;

  const char *what() const noexcept override;

private:
  Error() noexcept = default;
  friend impl<Error>;
  const char *msg;
  size_t len;
};
#endif // CXXBRIDGE1_RUST_ERROR

#ifndef CXXBRIDGE1_RUST_ISIZE
#define CXXBRIDGE1_RUST_ISIZE
#if defined(_WIN32)
using isize = SSIZE_T;
#else
using isize = ssize_t;
#endif
#endif // CXXBRIDGE1_RUST_ISIZE

std::ostream &operator<<(std::ostream &, const String &);
std::ostream &operator<<(std::ostream &, const Str &);

#ifndef CXXBRIDGE1_RUST_OPAQUE
#define CXXBRIDGE1_RUST_OPAQUE
// Base class of generated opaque Rust types.
class Opaque {
public:
  Opaque() = delete;
  Opaque(const Opaque &) = delete;
  ~Opaque() = delete;
};
#endif // CXXBRIDGE1_RUST_OPAQUE

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
template <typename T>
using slice = Slice<T>;
template <typename T>
using box = Box<T>;
template <typename T>
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

#ifndef CXXBRIDGE1_PANIC
#define CXXBRIDGE1_PANIC
template <typename Exception>
void panic [[noreturn]] (const char *msg);
#endif // CXXBRIDGE1_PANIC

#ifndef CXXBRIDGE1_RUST_FN
#define CXXBRIDGE1_RUST_FN
template <typename Ret, typename... Args, bool Throws>
Ret Fn<Ret(Args...), Throws>::operator()(Args... args) const noexcept(!Throws) {
  return (*this->trampoline)(std::move(args)..., this->fn);
}

template <typename Ret, typename... Args, bool Throws>
Fn<Ret(Args...), Throws> Fn<Ret(Args...), Throws>::operator*() const noexcept {
  return *this;
}
#endif // CXXBRIDGE1_RUST_FN

#ifndef CXXBRIDGE1_RUST_BITCOPY
#define CXXBRIDGE1_RUST_BITCOPY
struct unsafe_bitcopy_t final {
  explicit unsafe_bitcopy_t() = default;
};

constexpr unsafe_bitcopy_t unsafe_bitcopy{};
#endif // CXXBRIDGE1_RUST_BITCOPY

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
inline const char *Str::data() const noexcept { return this->ptr; }

inline size_t Str::size() const noexcept { return this->len; }

inline size_t Str::length() const noexcept { return this->len; }
#endif // CXXBRIDGE1_RUST_STR

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
template <typename T>
Slice<T>::Slice() noexcept : ptr(reinterpret_cast<T *>(alignof(T))), len(0) {}

template <typename T>
Slice<T>::Slice(T *s, size_t count) noexcept : ptr(s), len(count) {}

template <typename T>
T *Slice<T>::data() const noexcept {
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

template <typename T>
T &Slice<T>::iterator::operator*() const noexcept {
  return *this->pos;
}

template <typename T>
T *Slice<T>::iterator::operator->() const noexcept {
  return this->pos;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  ++this->pos;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  ++this->pos;
  return ret;
}

template <typename T>
bool Slice<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = this->ptr;
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos += this->len;
  return it;
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
#define CXXBRIDGE1_RUST_BOX
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
Box<T>::Box() noexcept = default;
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_VEC
#define CXXBRIDGE1_RUST_VEC
template <typename T>
Vec<T>::Vec(Vec &&other) noexcept : repr(other.repr) {
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
T &Vec<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
T *Vec<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Vec<T>::iterator &Vec<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Vec<T>::iterator Vec<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
bool Vec<T>::iterator::operator==(const iterator &other) const noexcept {
  return this->pos == other.pos;
}

template <typename T>
bool Vec<T>::iterator::operator!=(const iterator &other) const noexcept {
  return this->pos != other.pos;
}

template <typename T>
typename Vec<T>::iterator Vec<T>::begin() noexcept {
  iterator it;
  it.pos = const_cast<typename std::remove_const<T>::type *>(this->data());
  it.stride = this->stride();
  return it;
}

template <typename T>
typename Vec<T>::iterator Vec<T>::end() noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::begin() const noexcept {
  return this->cbegin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::end() const noexcept {
  return this->cend();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cbegin() const noexcept {
  const_iterator it;
  it.pos = const_cast<typename std::remove_const<T>::type *>(this->data());
  it.stride = this->stride();
  return it;
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cend() const noexcept {
  const_iterator it = this->cbegin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

// Internal API only intended for the cxxbridge code generator.
template <typename T>
Vec<T>::Vec(unsafe_bitcopy_t, const Vec &bits) noexcept : repr(bits.repr) {}
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_RELOCATABLE
#define CXXBRIDGE1_RELOCATABLE
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
#endif // CXXBRIDGE1_RELOCATABLE

} // namespace cxxbridge1
} // namespace rust
