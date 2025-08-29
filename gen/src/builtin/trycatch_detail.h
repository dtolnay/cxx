#pragma once
#include "./ptr_len.h"
#include <string>

namespace rust {
inline namespace cxxbridge1 {
namespace detail {
class Fail final {
  ::rust::repr::PtrLen &throw$;
  //
public:
  Fail(::rust::repr::PtrLen &throw$) noexcept : throw$(throw$) {}
  void operator()(char const *) noexcept;
  void operator()(std::string const &) noexcept;
};
} // namespace detail
} // namespace cxxbridge1
} // namespace rust
