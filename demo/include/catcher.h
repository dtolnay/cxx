#pragma once
#include <string>

namespace rust::behavior {
template <typename Try, typename Fail>
void trycatch(Try &&func, Fail &&fail) noexcept try {
  func();
} catch (int i) {
  auto msg = std::to_string(i);
  fail(msg.c_str());
}
}
