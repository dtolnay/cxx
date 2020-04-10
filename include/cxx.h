#pragma once
#include <array>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <iosfwd>
#include <string>
#include <type_traits>
#include <utility>

#ifdef _WIN32
#define ssize_t SSIZE_T
#endif

namespace rust {
inline namespace cxxbridge02 {

struct unsafe_bitcopy_t;

#ifndef CXXBRIDGE02_RUST_STRING
#define CXXBRIDGE02_RUST_STRING
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);

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
#endif // CXXBRIDGE02_RUST_STRING

#ifndef CXXBRIDGE02_RUST_STR
#define CXXBRIDGE02_RUST_STR
class Str final {
public:
  Str() noexcept;
  Str(const Str &) noexcept;

  Str(const std::string &);
  Str(const char *);
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
#endif // CXXBRIDGE02_RUST_STR

#ifndef CXXBRIDGE02_RUST_BOX
#define CXXBRIDGE02_RUST_BOX
template <typename T> class Box final {
public:
  using value_type = T;
  using const_pointer = typename std::add_pointer<
      typename std::add_const<value_type>::type>::type;
  using pointer = typename std::add_pointer<value_type>::type;

  Box(const Box &other) : Box(*other) {}
  Box(Box &&other) noexcept : ptr(other.ptr) { other.ptr = nullptr; }
  Box(const T &val) {
    this->uninit();
    ::new (this->ptr) T(val);
  }
  Box &operator=(const Box &other) {
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
  Box &operator=(Box &&other) noexcept {
    if (this->ptr) {
      this->drop();
    }
    this->ptr = other.ptr;
    other.ptr = nullptr;
    return *this;
  }
  ~Box() noexcept {
    if (this->ptr) {
      this->drop();
    }
  }

  const T *operator->() const noexcept { return this->ptr; }
  const T &operator*() const noexcept { return *this->ptr; }
  T *operator->() noexcept { return this->ptr; }
  T &operator*() noexcept { return *this->ptr; }

  // Important: requires that `raw` came from an into_raw call. Do not pass a
  // pointer from `new` or any other source.
  static Box from_raw(T *raw) noexcept {
    Box box;
    box.ptr = raw;
    return box;
  }

  T *into_raw() noexcept {
    T *raw = this->ptr;
    this->ptr = nullptr;
    return raw;
  }

private:
  Box() noexcept {}
  void uninit() noexcept;
  void drop() noexcept;
  T *ptr;
};
#endif // CXXBRIDGE02_RUST_BOX

#ifndef CXXBRIDGE02_RUST_FN
#define CXXBRIDGE02_RUST_FN
template <typename Signature, bool Throws = false> class Fn;

template <typename Ret, typename... Args, bool Throws>
class Fn<Ret(Args...), Throws> {
public:
  Ret operator()(Args... args) const noexcept(!Throws);
  Fn operator*() const noexcept;

private:
  Ret (*trampoline)(Args..., void *fn) noexcept(!Throws);
  void *fn;
};

template <typename Signature> using TryFn = Fn<Signature, true>;
#endif // CXXBRIDGE02_RUST_FN

#ifndef CXXBRIDGE02_RUST_ERROR
#define CXXBRIDGE02_RUST_ERROR
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
#endif // CXXBRIDGE02_RUST_ERROR

std::ostream &operator<<(std::ostream &, const String &);
std::ostream &operator<<(std::ostream &, const Str &);

// Snake case aliases for use in code that uses this style for type names.
using string = String;
using str = Str;
template <class T> using box = Box<T>;
using error = Error;
template <typename Signature, bool Throws = false>
using fn = Fn<Signature, Throws>;
template <typename Signature> using try_fn = TryFn<Signature>;

#ifndef CXXBRIDGE02_RUST_BITCOPY
#define CXXBRIDGE02_RUST_BITCOPY
struct unsafe_bitcopy_t {
  explicit unsafe_bitcopy_t() = default;
};
constexpr unsafe_bitcopy_t unsafe_bitcopy{};
#endif // CXXBRIDGE02_RUST_BITCOPY

template <typename Ret, typename... Args, bool Throws>
Ret Fn<Ret(Args...), Throws>::operator()(Args... args) const noexcept(!Throws) {
  return (*this->trampoline)(std::move(args)..., this->fn);
}

template <typename Ret, typename... Args, bool Throws>
Fn<Ret(Args...), Throws> Fn<Ret(Args...), Throws>::operator*() const noexcept {
  return *this;
}

} // namespace cxxbridge02
} // namespace rust
