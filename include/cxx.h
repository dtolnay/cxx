#pragma once
#include <algorithm>
#include <array>
#include <cassert>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <initializer_list>
#include <iosfwd>
#include <iterator>
#include <new>
#include <stdexcept>
#include <string>
#include <type_traits>
#include <utility>
// If you're using enums and variants on windows, you need to pass also
// `/Zc:__cplusplus` as a compiler to make __cplusplus work correctly. If users
// ever report that they have a too old compiler to `/Zc:__cplusplus` we may
// fallback to the `_MSVC_LANG` define.
//
// Sources:
// https://devblogs.microsoft.com/cppblog/msvc-now-correctly-reports-__cplusplus/
// https://learn.microsoft.com/en-us/cpp/build/reference/zc-cplusplus?view=msvc-170
#if __cplusplus >= 201703L
#include <variant>
#endif
#include <vector>
#if defined(_WIN32)
#include <basetsd.h>
#else
#include <sys/types.h>
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
// https://cxx.rs/binding/string.html
class String final {
public:
  String() noexcept;
  String(const String &) noexcept;
  String(String &&) noexcept;
  ~String() noexcept;

  String(const std::string &);
  String(const char *);
  String(const char *, std::size_t);
  String(const char16_t *);
  String(const char16_t *, std::size_t);

  // Replace invalid Unicode data with the replacement character (U+FFFD).
  static String lossy(const std::string &) noexcept;
  static String lossy(const char *) noexcept;
  static String lossy(const char *, std::size_t) noexcept;
  static String lossy(const char16_t *) noexcept;
  static String lossy(const char16_t *, std::size_t) noexcept;

  String &operator=(const String &) & noexcept;
  String &operator=(String &&) & noexcept;

  explicit operator std::string() const;

  // Note: no null terminator.
  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  const char *c_str() noexcept;

  std::size_t capacity() const noexcept;
  void reserve(size_t new_cap) noexcept;

  using iterator = char *;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const String &) const noexcept;
  bool operator!=(const String &) const noexcept;
  bool operator<(const String &) const noexcept;
  bool operator<=(const String &) const noexcept;
  bool operator>(const String &) const noexcept;
  bool operator>=(const String &) const noexcept;

  void swap(String &) noexcept;

  // Internal API only intended for the cxxbridge code generator.
  String(unsafe_bitcopy_t, const String &) noexcept;

private:
  struct lossy_t;
  String(lossy_t, const char *, std::size_t) noexcept;
  String(lossy_t, const char16_t *, std::size_t) noexcept;
  friend void swap(String &lhs, String &rhs) noexcept { lhs.swap(rhs); }

  // Size and alignment statically verified by rust_string.rs.
  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_STRING

#ifndef CXXBRIDGE1_RUST_STR
#define CXXBRIDGE1_RUST_STR
// https://cxx.rs/binding/str.html
class Str final {
public:
  Str() noexcept;
  Str(const String &) noexcept;
  Str(const std::string &);
  Str(const char *);
  Str(const char *, std::size_t);

  Str &operator=(const Str &) & noexcept = default;

  explicit operator std::string() const;

  // Note: no null terminator.
  const char *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Str(const Str &) noexcept = default;
  ~Str() noexcept = default;

  using iterator = const char *;
  using const_iterator = const char *;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  bool operator==(const Str &) const noexcept;
  bool operator!=(const Str &) const noexcept;
  bool operator<(const Str &) const noexcept;
  bool operator<=(const Str &) const noexcept;
  bool operator>(const Str &) const noexcept;
  bool operator>=(const Str &) const noexcept;

  void swap(Str &) noexcept;

private:
  class uninit;
  Str(uninit) noexcept;
  friend impl<Str>;

  std::array<std::uintptr_t, 2> repr;
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
  copy_assignable_if &operator=(const copy_assignable_if &) & noexcept = delete;
  copy_assignable_if &operator=(copy_assignable_if &&) & noexcept = default;
};
} // namespace detail

// https://cxx.rs/binding/slice.html
template <typename T>
class Slice final
    : private detail::copy_assignable_if<std::is_const<T>::value> {
public:
  using value_type = T;

  Slice() noexcept;
  Slice(T *, std::size_t count) noexcept;

  Slice &operator=(const Slice<T> &) & noexcept = default;
  Slice &operator=(Slice<T> &&) & noexcept = default;

  T *data() const noexcept;
  std::size_t size() const noexcept;
  std::size_t length() const noexcept;
  bool empty() const noexcept;

  T &operator[](std::size_t n) const noexcept;
  T &at(std::size_t n) const;
  T &front() const noexcept;
  T &back() const noexcept;

  // Important in order for System V ABI to pass in registers.
  Slice(const Slice<T> &) noexcept = default;
  ~Slice() noexcept = default;

  class iterator;
  iterator begin() const noexcept;
  iterator end() const noexcept;

  void swap(Slice &) noexcept;

private:
  class uninit;
  Slice(uninit) noexcept;
  friend impl<Slice>;
  friend void sliceInit(void *, const void *, std::size_t) noexcept;
  friend void *slicePtr(const void *) noexcept;
  friend std::size_t sliceLen(const void *) noexcept;

  std::array<std::uintptr_t, 2> repr;
};

template <typename T>
class Slice<T>::iterator final {
public:
  using iterator_category = std::random_access_iterator_tag;
  using value_type = T;
  using difference_type = std::ptrdiff_t;
  using pointer = typename std::add_pointer<T>::type;
  using reference = typename std::add_lvalue_reference<T>::type;

  reference operator*() const noexcept;
  pointer operator->() const noexcept;
  reference operator[](difference_type) const noexcept;

  iterator &operator++() noexcept;
  iterator operator++(int) noexcept;
  iterator &operator--() noexcept;
  iterator operator--(int) noexcept;

  iterator &operator+=(difference_type) noexcept;
  iterator &operator-=(difference_type) noexcept;
  iterator operator+(difference_type) const noexcept;
  iterator operator-(difference_type) const noexcept;
  difference_type operator-(const iterator &) const noexcept;

  bool operator==(const iterator &) const noexcept;
  bool operator!=(const iterator &) const noexcept;
  bool operator<(const iterator &) const noexcept;
  bool operator<=(const iterator &) const noexcept;
  bool operator>(const iterator &) const noexcept;
  bool operator>=(const iterator &) const noexcept;

private:
  friend class Slice;
  void *pos;
  std::size_t stride;
};
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
// https://cxx.rs/binding/box.html
template <typename T>
class Box final {
public:
  using element_type = T;
  using const_pointer =
      typename std::add_pointer<typename std::add_const<T>::type>::type;
  using pointer = typename std::add_pointer<T>::type;

  Box() = delete;
  Box(Box &&) noexcept;
  ~Box() noexcept;

  explicit Box(const T &);
  explicit Box(T &&);

  Box &operator=(Box &&) & noexcept;

  const T *operator->() const noexcept;
  const T &operator*() const noexcept;
  T *operator->() noexcept;
  T &operator*() noexcept;

  template <typename... Fields>
  static Box in_place(Fields &&...);

  void swap(Box &) noexcept;

  // Important: requires that `raw` came from an into_raw call. Do not pass a
  // pointer from `new` or any other source.
  static Box from_raw(T *) noexcept;

  T *into_raw() noexcept;

  /* Deprecated */ using value_type = element_type;

private:
  class uninit;
  class allocation;
  Box(uninit) noexcept;
  void drop() noexcept;

  friend void swap(Box &lhs, Box &rhs) noexcept { lhs.swap(rhs); }

  T *ptr;
};
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_VEC
// https://cxx.rs/binding/vec.html
template <typename T>
class Vec final {
public:
  using value_type = T;

  Vec() noexcept;
  Vec(std::initializer_list<T>);
  Vec(const Vec &);
  Vec(Vec &&) noexcept;
  ~Vec() noexcept;

  Vec &operator=(Vec &&) & noexcept;
  Vec &operator=(const Vec &) &;

  std::size_t size() const noexcept;
  bool empty() const noexcept;
  const T *data() const noexcept;
  T *data() noexcept;
  std::size_t capacity() const noexcept;

  const T &operator[](std::size_t n) const noexcept;
  const T &at(std::size_t n) const;
  const T &front() const noexcept;
  const T &back() const noexcept;

  T &operator[](std::size_t n) noexcept;
  T &at(std::size_t n);
  T &front() noexcept;
  T &back() noexcept;

  void reserve(std::size_t new_cap);
  void push_back(const T &value);
  void push_back(T &&value);
  template <typename... Args>
  void emplace_back(Args &&...args);
  void truncate(std::size_t len);
  void clear();

  using iterator = typename Slice<T>::iterator;
  iterator begin() noexcept;
  iterator end() noexcept;

  using const_iterator = typename Slice<const T>::iterator;
  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
  const_iterator cbegin() const noexcept;
  const_iterator cend() const noexcept;

  void swap(Vec &) noexcept;

  // Internal API only intended for the cxxbridge code generator.
  Vec(unsafe_bitcopy_t, const Vec &) noexcept;

private:
  void reserve_total(std::size_t new_cap) noexcept;
  void set_len(std::size_t len) noexcept;
  void drop() noexcept;

  friend void swap(Vec &lhs, Vec &rhs) noexcept { lhs.swap(rhs); }

  // Size and alignment statically verified by rust_vec.rs.
  std::array<std::uintptr_t, 3> repr;
};
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_RUST_VARIANT
#define CXXBRIDGE1_RUST_VARIANT

#if __cplusplus >= 201703L

// Adjusted from std::variant_alternative. Standard selects always the most
// specialized template specialization. See
// https://timsong-cpp.github.io/cppwp/n4140/temp.class.spec.match and
// https://timsong-cpp.github.io/cppwp/n4140/temp.class.order.
template <std::size_t I, typename... Ts>
struct variant_alternative;

// Specialization for gracefully handling invalid indices.
template <std::size_t I>
struct variant_alternative<I> {};

template <std::size_t I, typename First, typename... Remainder>
struct variant_alternative<I, First, Remainder...>
    : variant_alternative<I - 1, Remainder...> {};

template <typename First, typename... Remainder>
struct variant_alternative<0, First, Remainder...> {
  using type = First;
};

template <std::size_t I, typename... Ts>
using variant_alternative_t = typename variant_alternative<I, Ts...>::type;

template <bool... Values>
constexpr size_t compile_time_count() noexcept {
  return 0 + (static_cast<std::size_t>(Values) + ...);
}

template <bool... Values>
struct count
    : std::integral_constant<std::size_t, compile_time_count<Values...>()> {};

template <bool... Values>
struct exactly_once : std::conditional_t<count<Values...>::value == 1,
                                         std::true_type, std::false_type> {};

template <std::size_t I, bool... Values>
struct index_from_booleans;

template <std::size_t I>
struct index_from_booleans<I> {};

template <std::size_t I, bool First, bool... Remainder>
struct index_from_booleans<I, First, Remainder...>
    : index_from_booleans<I + 1, Remainder...> {};

template <std::size_t I, bool... Remainder>
struct index_from_booleans<I, true, Remainder...>
    : std::integral_constant<std::size_t, I> {};

template <typename Type, typename... Ts>
struct index_from_type
    : index_from_booleans<0, std::is_same_v<std::decay_t<Type>, Ts>...> {
  static_assert(exactly_once<std::is_same_v<std::decay_t<Type>, Ts>...>::value,
                "Index must be unique");
};

template <typename... Ts>
struct visitor_type;

template <typename... Ts>
struct variant_base;

template <std::size_t I, typename... Ts>
constexpr decltype(auto) get(variant_base<Ts...> &);

template <std::size_t I, typename... Ts>
constexpr decltype(auto) get(const variant_base<Ts...> &);

template <typename Visitor, typename... Ts>
constexpr decltype(auto) visit(Visitor &&visitor, variant_base<Ts...> &);

template <typename Visitor, typename... Ts>
constexpr decltype(auto) visit(Visitor &&visitor, const variant_base<Ts...> &);

/// @brief A std::variant like tagged union with the same memory layout as a
/// Rust Enum.
///
/// The memory layout of the Rust enum is defined under
/// https://doc.rust-lang.org/reference/type-layout.html#reprc-enums-with-fields
template <typename... Ts>
struct variant_base {
  static_assert(sizeof...(Ts) > 0,
                "variant_base must hold at least one alternative");

  /// @brief Delete the default constructor since we cannot be in an
  /// uninitialized state (if the first alternative throws). Corresponds to the
  /// (1) constructor in std::variant.
  variant_base() = delete;

  constexpr static bool all_copy_constructible_v =
      std::conjunction_v<std::is_copy_constructible<Ts>...>;

  /// @brief Copy constructor. Participates only in the resolution if all types
  /// are copy constructable. Corresponds to (2) constructor of std::variant.
  variant_base(const variant_base &other) {
    static_assert(
        all_copy_constructible_v,
        "Copy constructor requires that all types are copy constructable");

    m_Index = other.m_Index;
    visit(
        [this](const auto &value) {
          using type = std::decay_t<decltype(value)>;
          new (static_cast<void *>(m_Buff)) type(value);
        },
        other);
  };

  /// @brief Delete the move constructor since if we move this container it's
  /// unclear in which state it is. Corresponds to (3) constructor of
  /// std::variant.
  variant_base(variant_base &&other) = delete;

  template <typename T>
  constexpr static bool is_unique_v =
      exactly_once<std::is_same_v<Ts, std::decay_t<T>>...>::value;

  template <typename T>
  constexpr static std::size_t index_from_type_v =
      index_from_type<T, Ts...>::value;

  /// @brief Converting constructor. Corresponds to (4) constructor of
  /// std::variant.
  template <typename T, typename D = std::decay_t<T>,
            typename = std::enable_if_t<is_unique_v<T> &&
                                        std::is_constructible_v<D, T>>>
  variant_base(T &&other) noexcept(std::is_nothrow_constructible_v<D, T>) {
    m_Index = index_from_type_v<D>;
    new (static_cast<void *>(m_Buff)) D(std::forward<T>(other));
  }

  /// @brief Participates in the resolution only if we can construct T from Args
  /// and if T is unique in Ts. Corresponds to (5) constructor of std::variant.
  template <typename T, typename... Args,
            typename = std::enable_if_t<is_unique_v<T>>,
            typename = std::enable_if_t<std::is_constructible_v<T, Args...>>>
  explicit variant_base(std::in_place_type_t<T> type, Args &&...args) noexcept(
      std::is_nothrow_constructible_v<T, Args...>)
      : variant_base{std::in_place_index<index_from_type_v<T>>,
                     std::forward<Args>(args)...} {}

  template <std::size_t I>
  using type_from_index_t = variant_alternative_t<I, Ts...>;

  /// @brief Participates in the resolution only if the index is within range
  /// and if the type can be constructor from Args. Corresponds to (7) of
  /// std::variant.
  template <std::size_t I, typename... Args, typename T = type_from_index_t<I>,
            typename = std::enable_if_t<std::is_constructible_v<T, Args...>>>
  explicit variant_base(
      std::in_place_index_t<I> index,
      Args &&...args) noexcept(std::is_nothrow_constructible_v<T, Args...>) {
    m_Index = I;
    new (static_cast<void *>(m_Buff)) T(std::forward<Args>(args)...);
  }

  template <typename... Rs>
  constexpr static bool all_same_v =
      std::conjunction_v<std::is_same<Rs, Ts>...>;

  /// @brief Converts the std::variant to our variant. Participates only in
  /// the resolution if all types in Ts are copy constructable.
  template <typename... Rs, typename = std::enable_if_t<
                                all_same_v<Rs...> && all_copy_constructible_v>>
  variant_base(const std::variant<Rs...> &other) {
    m_Index = other.index();
    std::visit(
        [this](const auto &value) {
          using type = std::decay_t<decltype(value)>;
          new (static_cast<void *>(m_Buff)) type(value);
        },
        other);
  }

  constexpr static bool all_move_constructible_v =
      std::conjunction_v<std::is_move_constructible<Ts>...>;

  /// @brief Converts the std::variant to our variant. Participates only in
  /// the resolution if all types in Ts are move constructable.
  template <typename... Rs, typename = std::enable_if_t<
                                all_same_v<Rs...> && all_move_constructible_v>>
  variant_base(std::variant<Rs...> &&other) {
    m_Index = other.index();
    std::visit(
        [this](auto &&value) {
          using type = std::decay_t<decltype(value)>;
          new (static_cast<void *>(m_Buff)) type(std::move(value));
        },
        other);
  }

  ~variant_base() { destroy(); }

  /// @brief Copy assignment. Staticly fails if not every type in Ts is copy
  /// constructable. Corresponds to (1) assignment of std::variant.
  variant_base &operator=(const variant_base &other) {
    static_assert(
        all_copy_constructible_v,
        "Copy assignment requires that all types are copy constructable");

    visit([this](const auto &value) { *this = value; }, other);

    return *this;
  };

  /// @brief Deleted move assignment. Same as for the move constructor.
  /// Would correspond to (2) assignment of std::variant.
  variant_base &operator=(variant_base &&other) = delete;

  /// @brief Converting assignment. Corresponds to (3) assignment of
  /// std::variant.
  template <typename T, typename = std::enable_if_t<
                            is_unique_v<T> && std::is_constructible_v<T &&, T>>>
  variant_base &operator=(T &&other) {
    constexpr auto index = index_from_type_v<T>;

    if (m_Index == index) {
      if constexpr (std::is_nothrow_assignable_v<T, T &&>) {
        get<index>(*this) = std::forward<T>(other);
        return *this;
      }
    }
    this->emplace<std::decay_t<T>>(std::forward<T>(other));
    return *this;
  }

  /// @brief Converting assignment from std::variant. Participates only in the
  /// resolution if all types in Ts are copy constructable.
  template <typename... Rs, typename = std::enable_if_t<
                                all_same_v<Rs...> && all_copy_constructible_v>>
  variant_base &operator=(const std::variant<Rs...> &other) {
    // TODO this is not really clean since we fail if std::variant has
    // duplicated types.
    std::visit(
        [this](const auto &value) {
          using type = decltype(value);
          emplace<std::decay_t<type>>(value);
        },
        other);
    return *this;
  }

  /// @brief Converting assignment from std::variant. Participates only in the
  /// resolution if all types in Ts are move constructable.
  template <typename... Rs, typename = std::enable_if_t<
                                all_same_v<Rs...> && all_move_constructible_v>>
  variant_base &operator=(std::variant<Rs...> &&other) {
    // TODO this is not really clean since we fail if std::variant has
    // duplicated types.
    std::visit(
        [this](auto &&value) {
          using type = decltype(value);
          emplace<std::decay_t<type>>(std::move(value));
        },
        other);
    return *this;
  }

  /// @brief Emplace function. Participates in the resolution only if T is
  /// unique in Ts and if T can be constructed from Args. Offers strong
  /// exception guarantee. Corresponds to the (1) emplace function of
  /// std::variant.
  template <typename T, typename... Args,
            typename = std::enable_if_t<is_unique_v<T>>,
            typename = std::enable_if_t<std::is_constructible_v<T, Args...>>>
  T &emplace(Args &&...args) {
    constexpr std::size_t index = index_from_type_v<T>;
    return this->emplace<index>(std::forward<Args>(args)...);
  }

  /// @brief Emplace function. Participates in the resolution only if T can be
  /// constructed from Args. Offers strong exception guarantee. Corresponds to
  /// the (2) emplace function of std::variant.
  ///
  /// The std::variant can have no valid state if the type throws during the
  /// construction. This is represented by the `valueless_by_exception` flag.
  /// The same approach is also used in absl::variant [2].
  /// In our case we can't accept valueless enums since we can't represent this
  /// in Rust. We must therefore provide a strong exception guarantee for all
  /// operations using `emplace`. Two famous implementations of never valueless
  /// variants are Boost/variant [3] and Boost/variant2 [4]. Boost/variant2 uses
  /// two memory buffers - which would be not compatible with Rust Enum's memory
  /// layout. The Boost/variant backs up the old object and calls its d'tor
  /// before constructing the new object; It then copies the old data back to
  /// the buffer if the construction fails - which might contain garbage (since)
  /// the d'tor was already called.
  ///
  ///
  /// We take a similar approach to Boost/variant. Assuming that constructing or
  /// moving the new type can throw, we backup the old data, try to construct
  /// the new object in the final buffer, swap the buffers, such that the old
  /// object is back in its original place, detroy it and move the new object
  /// from the old buffer back to the final place.
  ///
  /// Sources
  ///
  /// [1]
  /// https://en.cppreference.com/w/cpp/utility/variant/valueless_by_exception
  /// [2]
  /// https://github.com/abseil/abseil-cpp/blob/master/absl/types/variant.h
  /// [3]
  /// https://www.boost.org/doc/libs/1_84_0/libs/variant2/doc/html/variant2.html
  /// [4]
  /// https://www.boost.org/doc/libs/1_84_0/doc/html/variant/design.html#variant.design.never-empty
  template <std::size_t I, typename... Args, typename T = type_from_index_t<I>,
            typename = std::enable_if_t<std::is_constructible_v<T, Args...>>>
  T &emplace(Args &&...args) {
    if constexpr (std::is_nothrow_constructible_v<T, Args...>) {
      destroy();
      new (static_cast<void *>(m_Buff)) T(std::forward<Args>(args)...);
    } else if constexpr (std::is_nothrow_move_constructible_v<T>) {
      // This operation may throw, but we know that the move does not.
      const T tmp{std::forward<Args>(args)...};

      // The operations below are save.
      destroy();
      new (static_cast<void *>(m_Buff)) T(std::move(tmp));
    } else {
      // Backup the old data.
      alignas(Ts...) std::byte old_buff[std::max({sizeof(Ts)...})];
      std::memcpy(old_buff, m_Buff, sizeof(m_Buff));

      try {
        // Try to construct the new object
        new (static_cast<void *>(m_Buff)) T(std::forward<Args>(args)...);
      } catch (...) {
        // Restore the old buffer
        std::memcpy(m_Buff, old_buff, sizeof(m_Buff));
        throw;
      }
      // Fetch the old buffer and detroy it.
      std::swap_ranges(m_Buff, m_Buff + sizeof(m_Buff), old_buff);

      destroy();
      std::memcpy(m_Buff, old_buff, sizeof(m_Buff));
    }

    m_Index = I;
    return get<I>(*this);
  }

  constexpr std::size_t index() const noexcept { return m_Index; }
  void swap(variant_base &other) {
    // TODO
  }

  struct my_bad_variant_access : std::runtime_error {
    my_bad_variant_access(std::size_t index)
        : std::runtime_error{"The index should be " + std::to_string(index)} {}
  };

private:
  template <std::size_t I>
  void throw_if_invalid() const {
    static_assert(I < (sizeof...(Ts)), "Invalid index");

    if (m_Index != I)
      throw my_bad_variant_access(m_Index);
  }

  void destroy() {
    visit(
        [](const auto &value) {
          using type = std::decay_t<decltype(value)>;
          value.~type();
        },
        *this);
  }

  // The underlying type is not fixed, but should be int - which we will verify
  // statically. See
  // https://timsong-cpp.github.io/cppwp/n4659/dcl.enum#7
  int m_Index;

  // std::aligned_storage is deprecated and may be replaced with the construct
  // below. See
  // https://www.open-std.org/jtc1/sc22/wg21/docs/papers/2021/p1413r3.pdf
  alignas(Ts...) std::byte m_Buff[std::max({sizeof(Ts)...})];

  // The friend zone
  template <std::size_t I, typename... Rs>
  friend constexpr decltype(auto) get(variant_base<Rs...> &variant);

  template <std::size_t I, typename... Rs>
  friend constexpr decltype(auto) get(const variant_base<Rs...> &variant);

  template <typename... Rs>
  friend struct visitor_type;
};

template <typename First, typename... Remainder>
struct visitor_type<First, Remainder...> {
  template <typename Visitor, typename Variant>
  constexpr static decltype(auto) visit(Visitor &&visitor, Variant &&variant) {
    return visit(std::forward<Visitor>(visitor), variant.m_Index,
                 variant.m_Buff);
  }

  /// @brief The visit method which will pick the right type depending on the
  /// `index` value.
  template <typename Visitor>
  constexpr static auto visit(Visitor &&visitor, std::size_t index,
                              std::byte *data)
      -> decltype(visitor(*reinterpret_cast<First *>(data))) {
    if (index == 0) {
      return visitor(*reinterpret_cast<First *>(data));
    }
    if constexpr (sizeof...(Remainder) != 0) {
      return visitor_type<Remainder...>::visit(std::forward<Visitor>(visitor),
                                               --index, data);
    }
    throw std::out_of_range("invalid");
  }

  template <typename Visitor>
  constexpr static auto visit(Visitor &&visitor, std::size_t index,
                              const std::byte *data)
      -> decltype(visitor(*reinterpret_cast<const First *>(data))) {
    if (index == 0) {
      return visitor(*reinterpret_cast<const First *>(data));
    }
    if constexpr (sizeof...(Remainder) != 0) {
      return visitor_type<Remainder...>::visit(std::forward<Visitor>(visitor),
                                               --index, data);
    }
    throw std::out_of_range("invalid");
  }
};

/// @brief Applies the visitor to the variant. Corresponds to the (3)
/// std::visit defintion.
template <typename Visitor, typename... Ts>
constexpr decltype(auto) visit(Visitor &&visitor,
                               variant_base<Ts...> &variant) {
  return visitor_type<Ts...>::visit(std::forward<Visitor>(visitor), variant);
}

/// @brief Applies the visitor to the variant. Corresponds to the (4)
/// std::visit defintion.
template <typename Visitor, typename... Ts>
constexpr decltype(auto) visit(Visitor &&visitor,
                               const variant_base<Ts...> &variant) {
  return visitor_type<Ts...>::visit(std::forward<Visitor>(visitor), variant);
}

template <std::size_t I, typename... Ts>
constexpr decltype(auto) get(variant_base<Ts...> &variant) {
  variant.template throw_if_invalid<I>();
  return *reinterpret_cast<variant_alternative_t<I, Ts...> *>(variant.m_Buff);
}

template <std::size_t I, typename... Ts>
constexpr decltype(auto) get(const variant_base<Ts...> &variant) {
  variant.template throw_if_invalid<I>();
  return *reinterpret_cast<const variant_alternative_t<I, Ts...> *>(
      variant.m_Buff);
}

template <typename T, typename... Ts,
          typename = std::enable_if_t<
              exactly_once<std::is_same_v<Ts, std::decay_t<T>>...>::value>>
constexpr const T &get(const variant_base<Ts...> &variant) {
  constexpr auto index = index_from_type<T, Ts...>::value;
  return get<index>(variant);
}

template <typename T, typename... Ts,
          typename = std::enable_if_t<
              exactly_once<std::is_same_v<Ts, std::decay_t<T>>...>::value>>
constexpr T &get(variant_base<Ts...> &variant) {
  constexpr auto index = index_from_type<T, Ts...>::value;
  return get<index>(variant);
}

template <std::size_t I, typename... Ts>
constexpr bool holds_alternative(const variant_base<Ts...> &variant) {
  return variant.index() == I;
}

template <typename T, typename... Ts,
          typename = std::enable_if_t<
              exactly_once<std::is_same_v<Ts, std::decay_t<T>>...>::value>>
constexpr bool holds_alternative(const variant_base<Ts...> &variant) {
  return variant.index() == index_from_type<T, Ts...>::value;
}

template <bool>
struct copy_control;

template <>
struct copy_control<true> {
  copy_control() = default;
  copy_control(const copy_control &other) = default;
  copy_control &operator=(const copy_control &) = default;
};

template <>
struct copy_control<false> {
  copy_control() = default;
  copy_control(const copy_control &other) = delete;
  copy_control &operator=(const copy_control &) = delete;
};

template <typename... Ts>
using allow_copy =
    copy_control<std::conjunction_v<std::is_copy_constructible<Ts>...>>;

template <typename... Ts>
struct variant : public variant_base<Ts...>, private allow_copy<Ts...> {
  using base = variant_base<Ts...>;

  variant() = delete;
  variant(const variant &) = default;
  variant(variant &&) = delete;

  using base::base;

  variant &operator=(const variant &) = default;
  variant &operator=(variant &&) = delete;

  using base::operator=;
};

/// An empty type used for unit variants from Rust.
struct empty {};

#endif

#endif

#ifndef CXXBRIDGE1_RUST_FN
// https://cxx.rs/binding/fn.html
template <typename Signature>
class Fn;

template <typename Ret, typename... Args>
class Fn<Ret(Args...)> final {
public:
  Ret operator()(Args... args) const noexcept;
  Fn operator*() const noexcept;

private:
  Ret (*trampoline)(Args..., void *fn) noexcept;
  void *fn;
};
#endif // CXXBRIDGE1_RUST_FN

#ifndef CXXBRIDGE1_RUST_ERROR
#define CXXBRIDGE1_RUST_ERROR
// https://cxx.rs/binding/result.html
class Error final : public std::exception {
public:
  Error(const Error &);
  Error(Error &&) noexcept;
  ~Error() noexcept override;

  Error &operator=(const Error &) &;
  Error &operator=(Error &&) & noexcept;

  const char *what() const noexcept override;

private:
  Error() noexcept = default;
  friend impl<Error>;
  const char *msg;
  std::size_t len;
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

template <typename T>
std::size_t size_of();
template <typename T>
std::size_t align_of();

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

using u8 = std::uint8_t;
using u16 = std::uint16_t;
using u32 = std::uint32_t;
using u64 = std::uint64_t;
using usize = std::size_t; // see static asserts in cxx.cc
using i8 = std::int8_t;
using i16 = std::int16_t;
using i32 = std::int32_t;
using i64 = std::int64_t;
using f32 = float;
using f64 = double;

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
template <typename Signature>
using fn = Fn<Signature>;
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
template <typename Ret, typename... Args>
Ret Fn<Ret(Args...)>::operator()(Args... args) const noexcept {
  return (*this->trampoline)(std::forward<Args>(args)..., this->fn);
}

template <typename Ret, typename... Args>
Fn<Ret(Args...)> Fn<Ret(Args...)>::operator*() const noexcept {
  return *this;
}
#endif // CXXBRIDGE1_RUST_FN

#ifndef CXXBRIDGE1_RUST_BITCOPY_T
#define CXXBRIDGE1_RUST_BITCOPY_T
struct unsafe_bitcopy_t final {
  explicit unsafe_bitcopy_t() = default;
};
#endif // CXXBRIDGE1_RUST_BITCOPY_T

#ifndef CXXBRIDGE1_RUST_BITCOPY
#define CXXBRIDGE1_RUST_BITCOPY
constexpr unsafe_bitcopy_t unsafe_bitcopy{};
#endif // CXXBRIDGE1_RUST_BITCOPY

#ifndef CXXBRIDGE1_RUST_SLICE
#define CXXBRIDGE1_RUST_SLICE
template <typename T>
Slice<T>::Slice() noexcept {
  sliceInit(this, reinterpret_cast<void *>(align_of<T>()), 0);
}

template <typename T>
Slice<T>::Slice(T *s, std::size_t count) noexcept {
  assert(s != nullptr || count == 0);
  sliceInit(this,
            s == nullptr && count == 0
                ? reinterpret_cast<void *>(align_of<T>())
                : const_cast<typename std::remove_const<T>::type *>(s),
            count);
}

template <typename T>
T *Slice<T>::data() const noexcept {
  return reinterpret_cast<T *>(slicePtr(this));
}

template <typename T>
std::size_t Slice<T>::size() const noexcept {
  return sliceLen(this);
}

template <typename T>
std::size_t Slice<T>::length() const noexcept {
  return this->size();
}

template <typename T>
bool Slice<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T &Slice<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto ptr = static_cast<char *>(slicePtr(this)) + size_of<T>() * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
T &Slice<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Slice index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Slice<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Slice<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
typename Slice<T>::iterator::reference
Slice<T>::iterator::operator*() const noexcept {
  return *static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::pointer
Slice<T>::iterator::operator->() const noexcept {
  return static_cast<T *>(this->pos);
}

template <typename T>
typename Slice<T>::iterator::reference Slice<T>::iterator::operator[](
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ptr = static_cast<char *>(this->pos) + this->stride * n;
  return *reinterpret_cast<T *>(ptr);
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator++() noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator++(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) + this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator--() noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator--(int) noexcept {
  auto ret = iterator(*this);
  this->pos = static_cast<char *>(this->pos) - this->stride;
  return ret;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator+=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) + this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator &Slice<T>::iterator::operator-=(
    typename Slice<T>::iterator::difference_type n) noexcept {
  this->pos = static_cast<char *>(this->pos) - this->stride * n;
  return *this;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator+(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) + this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::iterator::operator-(
    typename Slice<T>::iterator::difference_type n) const noexcept {
  auto ret = iterator(*this);
  ret.pos = static_cast<char *>(this->pos) - this->stride * n;
  return ret;
}

template <typename T>
typename Slice<T>::iterator::difference_type
Slice<T>::iterator::operator-(const iterator &other) const noexcept {
  auto diff = std::distance(static_cast<char *>(other.pos),
                            static_cast<char *>(this->pos));
  return diff / static_cast<typename Slice<T>::iterator::difference_type>(
                    this->stride);
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
bool Slice<T>::iterator::operator<(const iterator &other) const noexcept {
  return this->pos < other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator<=(const iterator &other) const noexcept {
  return this->pos <= other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>(const iterator &other) const noexcept {
  return this->pos > other.pos;
}

template <typename T>
bool Slice<T>::iterator::operator>=(const iterator &other) const noexcept {
  return this->pos >= other.pos;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::begin() const noexcept {
  iterator it;
  it.pos = slicePtr(this);
  it.stride = size_of<T>();
  return it;
}

template <typename T>
typename Slice<T>::iterator Slice<T>::end() const noexcept {
  iterator it = this->begin();
  it.pos = static_cast<char *>(it.pos) + it.stride * this->size();
  return it;
}

template <typename T>
void Slice<T>::swap(Slice &rhs) noexcept {
  std::swap(*this, rhs);
}
#endif // CXXBRIDGE1_RUST_SLICE

#ifndef CXXBRIDGE1_RUST_BOX
#define CXXBRIDGE1_RUST_BOX
template <typename T>
class Box<T>::uninit {};

template <typename T>
class Box<T>::allocation {
  static T *alloc() noexcept;
  static void dealloc(T *) noexcept;

public:
  allocation() noexcept : ptr(alloc()) {}
  ~allocation() noexcept {
    if (this->ptr) {
      dealloc(this->ptr);
    }
  }
  T *ptr;
};

template <typename T>
Box<T>::Box(Box &&other) noexcept : ptr(other.ptr) {
  other.ptr = nullptr;
}

template <typename T>
Box<T>::Box(const T &val) {
  allocation alloc;
  ::new (alloc.ptr) T(val);
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::Box(T &&val) {
  allocation alloc;
  ::new (alloc.ptr) T(std::move(val));
  this->ptr = alloc.ptr;
  alloc.ptr = nullptr;
}

template <typename T>
Box<T>::~Box() noexcept {
  if (this->ptr) {
    this->drop();
  }
}

template <typename T>
Box<T> &Box<T>::operator=(Box &&other) & noexcept {
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
Box<T> Box<T>::in_place(Fields &&...fields) {
  allocation alloc;
  auto ptr = alloc.ptr;
  ::new (ptr) T{std::forward<Fields>(fields)...};
  alloc.ptr = nullptr;
  return from_raw(ptr);
}

template <typename T>
void Box<T>::swap(Box &rhs) noexcept {
  using std::swap;
  swap(this->ptr, rhs.ptr);
}

template <typename T>
Box<T> Box<T>::from_raw(T *raw) noexcept {
  Box box = uninit{};
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
Box<T>::Box(uninit) noexcept {}
#endif // CXXBRIDGE1_RUST_BOX

#ifndef CXXBRIDGE1_RUST_VEC
#define CXXBRIDGE1_RUST_VEC
template <typename T>
Vec<T>::Vec(std::initializer_list<T> init) : Vec{} {
  this->reserve_total(init.size());
  std::move(init.begin(), init.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(const Vec &other) : Vec() {
  this->reserve_total(other.size());
  std::copy(other.begin(), other.end(), std::back_inserter(*this));
}

template <typename T>
Vec<T>::Vec(Vec &&other) noexcept : repr(other.repr) {
  new (&other) Vec();
}

template <typename T>
Vec<T>::~Vec() noexcept {
  this->drop();
}

template <typename T>
Vec<T> &Vec<T>::operator=(Vec &&other) & noexcept {
  this->drop();
  this->repr = other.repr;
  new (&other) Vec();
  return *this;
}

template <typename T>
Vec<T> &Vec<T>::operator=(const Vec &other) & {
  if (this != &other) {
    this->drop();
    new (this) Vec(other);
  }
  return *this;
}

template <typename T>
bool Vec<T>::empty() const noexcept {
  return this->size() == 0;
}

template <typename T>
T *Vec<T>::data() noexcept {
  return const_cast<T *>(const_cast<const Vec<T> *>(this)->data());
}

template <typename T>
const T &Vec<T>::operator[](std::size_t n) const noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<const char *>(this->data());
  return *reinterpret_cast<const T *>(data + n * size_of<T>());
}

template <typename T>
const T &Vec<T>::at(std::size_t n) const {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
const T &Vec<T>::front() const noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
const T &Vec<T>::back() const noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
T &Vec<T>::operator[](std::size_t n) noexcept {
  assert(n < this->size());
  auto data = reinterpret_cast<char *>(this->data());
  return *reinterpret_cast<T *>(data + n * size_of<T>());
}

template <typename T>
T &Vec<T>::at(std::size_t n) {
  if (n >= this->size()) {
    panic<std::out_of_range>("rust::Vec index out of range");
  }
  return (*this)[n];
}

template <typename T>
T &Vec<T>::front() noexcept {
  assert(!this->empty());
  return (*this)[0];
}

template <typename T>
T &Vec<T>::back() noexcept {
  assert(!this->empty());
  return (*this)[this->size() - 1];
}

template <typename T>
void Vec<T>::reserve(std::size_t new_cap) {
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
void Vec<T>::emplace_back(Args &&...args) {
  auto size = this->size();
  this->reserve_total(size + 1);
  ::new (reinterpret_cast<T *>(reinterpret_cast<char *>(this->data()) +
                               size * size_of<T>()))
      T(std::forward<Args>(args)...);
  this->set_len(size + 1);
}

template <typename T>
void Vec<T>::clear() {
  this->truncate(0);
}

template <typename T>
typename Vec<T>::iterator Vec<T>::begin() noexcept {
  return Slice<T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::iterator Vec<T>::end() noexcept {
  return Slice<T>(this->data(), this->size()).end();
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
  return Slice<const T>(this->data(), this->size()).begin();
}

template <typename T>
typename Vec<T>::const_iterator Vec<T>::cend() const noexcept {
  return Slice<const T>(this->data(), this->size()).end();
}

template <typename T>
void Vec<T>::swap(Vec &rhs) noexcept {
  using std::swap;
  swap(this->repr, rhs.repr);
}

// Internal API only intended for the cxxbridge code generator.
template <typename T>
Vec<T>::Vec(unsafe_bitcopy_t, const Vec &bits) noexcept : repr(bits.repr) {}
#endif // CXXBRIDGE1_RUST_VEC

#ifndef CXXBRIDGE1_IS_COMPLETE
#define CXXBRIDGE1_IS_COMPLETE
namespace detail {
namespace {
template <typename T, typename = std::size_t>
struct is_complete : std::false_type {};
template <typename T>
struct is_complete<T, decltype(sizeof(T))> : std::true_type {};
} // namespace
} // namespace detail
#endif // CXXBRIDGE1_IS_COMPLETE

#ifndef CXXBRIDGE1_LAYOUT
#define CXXBRIDGE1_LAYOUT
class layout {
  template <typename T>
  friend std::size_t size_of();
  template <typename T>
  friend std::size_t align_of();
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return T::layout::size();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_size_of() {
    return sizeof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      size_of() {
    return do_size_of<T>();
  }
  template <typename T>
  static typename std::enable_if<std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return T::layout::align();
  }
  template <typename T>
  static typename std::enable_if<!std::is_base_of<Opaque, T>::value,
                                 std::size_t>::type
  do_align_of() {
    return alignof(T);
  }
  template <typename T>
  static
      typename std::enable_if<detail::is_complete<T>::value, std::size_t>::type
      align_of() {
    return do_align_of<T>();
  }
};

template <typename T>
std::size_t size_of() {
  return layout::size_of<T>();
}

template <typename T>
std::size_t align_of() {
  return layout::align_of<T>();
}
#endif // CXXBRIDGE1_LAYOUT

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
