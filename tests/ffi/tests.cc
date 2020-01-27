#include "tests/ffi/tests.h"
#include "tests/ffi/lib.rs"

extern "C" void cxx_test_suite_set_correct();

namespace tests {

C::C(size_t n) : n(n) {}

size_t C::get() const { return this->n; }

size_t c_return_primitive() { return 2020; }

Shared c_return_shared() { return Shared{2020}; }

std::unique_ptr<C> c_return_unique_ptr() {
  return std::unique_ptr<C>(new C{2020});
}

const size_t &c_return_ref(const Shared &shared) { return shared.z; }

cxxbridge::RustStr c_return_str(const Shared &shared) {
  (void)shared;
  return "2020";
}

cxxbridge::RustString c_return_rust_string() { return "2020"; }

std::unique_ptr<std::string> c_return_unique_ptr_string() {
  return std::unique_ptr<std::string>(new std::string("2020"));
}

void c_take_primitive(size_t n) {
  if (n == 2020) {
    cxx_test_suite_set_correct();
  }
}

void c_take_shared(Shared shared) {
  if (shared.z == 2020) {
    cxx_test_suite_set_correct();
  }
}

void c_take_box(cxxbridge::RustBox<R> r) {
  (void)r;
  cxx_test_suite_set_correct();
}

void c_take_unique_ptr(std::unique_ptr<C> c) {
  if (c->get() == 2020) {
    cxx_test_suite_set_correct();
  }
}

void c_take_ref_r(const R &r) { (void)r; }

void c_take_ref_c(const C &c) {
  if (c.get() == 2020) {
    cxx_test_suite_set_correct();
  }
}

void c_take_str(cxxbridge::RustStr s) {
  if (std::string(s) == "2020") {
    cxx_test_suite_set_correct();
  }
}

void c_take_rust_string(cxxbridge::RustString s) {
  if (std::string(s) == "2020") {
    cxx_test_suite_set_correct();
  }
}

void c_take_unique_ptr_string(std::unique_ptr<std::string> s) {
  if (*s == "2020") {
    cxx_test_suite_set_correct();
  }
}

} // namespace tests
