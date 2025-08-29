#pragma once
#include <utility>

namespace rust {
inline namespace cxxbridge1 {
template <typename T>
union ManuallyDrop {
  T value;
  ManuallyDrop(T &&value) : value(::std::move(value)) {}
  ~ManuallyDrop() {}
};
} // namespace cxxbridge1
} // namespace rust
