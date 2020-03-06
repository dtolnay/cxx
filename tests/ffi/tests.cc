#include "tests/ffi/tests.h"
#include "tests/ffi/lib.rs"

extern "C" void cxx_test_suite_set_correct();
extern "C" tests::R *cxx_test_suite_get_box();

namespace tests {

C::C(size_t n) : n(n) {}

size_t C::get() const { return this->n; }

size_t c_return_primitive() { return 2020; }

Shared c_return_shared() { return Shared{2020}; }

rust::Box<R> c_return_box() {
  return rust::Box<R>::from_raw(cxx_test_suite_get_box());
}

std::unique_ptr<C> c_return_unique_ptr() {
  return std::unique_ptr<C>(new C{2020});
}

const size_t &c_return_ref(const Shared &shared) { return shared.z; }

rust::Str c_return_str(const Shared &shared) {
  (void)shared;
  return "2020";
}

rust::String c_return_rust_string() { return "2020"; }

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

void c_take_box(rust::Box<R> r) {
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

void c_take_str(rust::Str s) {
  if (std::string(s) == "2020") {
    cxx_test_suite_set_correct();
  }
}

void c_take_rust_string(rust::String s) {
  if (std::string(s) == "2020") {
    cxx_test_suite_set_correct();
  }
}

void c_take_unique_ptr_string(std::unique_ptr<std::string> s) {
  if (*s == "2020") {
    cxx_test_suite_set_correct();
  }
}

extern "C" const char *cxx_run_test() noexcept {
#define STRINGIFY(x) #x
#define TOSTRING(x) STRINGIFY(x)
#define ASSERT(x)                                                              \
  do {                                                                         \
    if (!(x)) {                                                                \
      return "Assertion failed: `" #x "`, " __FILE__ ":" TOSTRING(__LINE__);   \
    }                                                                          \
  } while (false)

  ASSERT(r_return_primitive() == 2020);
  ASSERT(r_return_shared().z == 2020);
  ASSERT(r_return_ref(Shared{2020}) == 2020);
  ASSERT(std::string(r_return_str(Shared{2020})) == "2020");
  ASSERT(std::string(r_return_rust_string()) == "2020");

  r_take_primitive(2020);
  r_take_shared(Shared{2020});
  r_take_unique_ptr(std::unique_ptr<C>(new C{2020}));
  r_take_ref_c(C{2020});
  r_take_str(rust::Str("2020"));
  r_take_rust_string(rust::String("2020"));
  r_take_unique_ptr_string(
      std::unique_ptr<std::string>(new std::string("2020")));

  cxx_test_suite_set_correct();
  return nullptr;
}

} // namespace tests
