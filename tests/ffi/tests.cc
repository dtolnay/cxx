#include "tests/ffi/tests.h"
#include "tests/ffi/lib.rs.h"
#include <cstring>
#include <numeric>
#include <stdexcept>

extern "C" void cxx_test_suite_set_correct() noexcept;
extern "C" tests::R *cxx_test_suite_get_box() noexcept;
extern "C" bool cxx_test_suite_r_is_correct(const tests::R *) noexcept;

namespace tests {

static constexpr char SLICE_DATA[] = "2020";

C::C(size_t n) : n(n) {}

size_t C::get() const { return this->n; }

size_t C::get2() const { return this->n; }

size_t C::set(size_t n) {
  this->n = n;
  return this->n;
}

size_t C::set2(size_t n) {
  this->n = n;
  return this->n;
}

size_t C::set_succeed(size_t n) { return this->set2(n); }

size_t C::get_fail() { throw std::runtime_error("unimplemented"); }

const std::vector<uint8_t> &C::get_v() const { return this->v; }

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

rust::Slice<uint8_t> c_return_sliceu8(const Shared &shared) {
  (void)shared;
  return rust::Slice<uint8_t>(reinterpret_cast<const uint8_t *>(SLICE_DATA),
                              sizeof(SLICE_DATA));
}

rust::String c_return_rust_string() { return "2020"; }

std::unique_ptr<std::string> c_return_unique_ptr_string() {
  return std::unique_ptr<std::string>(new std::string("2020"));
}

std::unique_ptr<std::vector<uint8_t>> c_return_unique_ptr_vector_u8() {
  auto vec = std::unique_ptr<std::vector<uint8_t>>(new std::vector<uint8_t>());
  vec->push_back(86);
  vec->push_back(75);
  vec->push_back(30);
  vec->push_back(9);
  return vec;
}

std::unique_ptr<std::vector<double>> c_return_unique_ptr_vector_f64() {
  auto vec = std::unique_ptr<std::vector<double>>(new std::vector<double>());
  vec->push_back(86.0);
  vec->push_back(75.0);
  vec->push_back(30.0);
  vec->push_back(9.5);
  return vec;
}

std::unique_ptr<std::vector<Shared>> c_return_unique_ptr_vector_shared() {
  auto vec = std::unique_ptr<std::vector<Shared>>(new std::vector<Shared>());
  vec->push_back(Shared{1010});
  vec->push_back(Shared{1011});
  return vec;
}

std::unique_ptr<std::vector<C>> c_return_unique_ptr_vector_opaque() {
  return std::unique_ptr<std::vector<C>>(new std::vector<C>());
}

const std::vector<uint8_t> &c_return_ref_vector(const C &c) {
  return c.get_v();
}

rust::Vec<uint8_t> c_return_rust_vec() {
  throw std::runtime_error("unimplemented");
}

const rust::Vec<uint8_t> &c_return_ref_rust_vec(const C &c) {
  (void)c;
  throw std::runtime_error("unimplemented");
}

size_t c_return_identity(size_t n) { return n; }

size_t c_return_sum(size_t n1, size_t n2) { return n1 + n2; }

Enum c_return_enum(uint16_t n) {
  if (n <= static_cast<uint16_t>(Enum::AVal)) {
    return Enum::AVal;
  } else if (n <= static_cast<uint16_t>(Enum::BVal)) {
    return Enum::BVal;
  } else {
    return Enum::CVal;
  }
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
  if (cxx_test_suite_r_is_correct(&*r)) {
    cxx_test_suite_set_correct();
  }
}

void c_take_unique_ptr(std::unique_ptr<C> c) {
  if (c->get() == 2020) {
    cxx_test_suite_set_correct();
  }
}

void c_take_ref_r(const R &r) {
  if (cxx_test_suite_r_is_correct(&r)) {
    cxx_test_suite_set_correct();
  }
}

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

void c_take_sliceu8(rust::Slice<uint8_t> s) {
  if (std::string(reinterpret_cast<const char *>(s.data()), s.size()) ==
      "2020") {
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

void c_take_unique_ptr_vector_u8(std::unique_ptr<std::vector<uint8_t>> v) {
  if (v->size() == 4) {
    cxx_test_suite_set_correct();
  }
}

void c_take_unique_ptr_vector_f64(std::unique_ptr<std::vector<double>> v) {
  if (v->size() == 4) {
    cxx_test_suite_set_correct();
  }
}

void c_take_unique_ptr_vector_shared(std::unique_ptr<std::vector<Shared>> v) {
  if (v->size() == 2) {
    cxx_test_suite_set_correct();
  }
}

void c_take_ref_vector(const std::vector<uint8_t> &v) {
  if (v.size() == 4) {
    cxx_test_suite_set_correct();
  }
}

void c_take_rust_vec(rust::Vec<uint8_t> v) { c_take_ref_rust_vec(v); }

void c_take_rust_vec_shared(rust::Vec<Shared> v) {
  uint32_t sum = 0;
  for (auto i : v) {
    sum += i.z;
  }
  if (sum == 2021) {
    cxx_test_suite_set_correct();
  }
}

void c_take_rust_vec_shared_forward_iterator(rust::Vec<Shared> v) {
  // Exercise requirements of ForwardIterator
  // https://en.cppreference.com/w/cpp/named_req/ForwardIterator
  uint32_t sum = 0;
  for (auto it = v.begin(), it_end = v.end(); it != it_end; it++) {
    sum += it->z;
  }
  if (sum == 2021) {
    cxx_test_suite_set_correct();
  }
}

void c_take_ref_rust_vec(const rust::Vec<uint8_t> &v) {
  uint8_t sum = std::accumulate(v.begin(), v.end(), 0);
  if (sum == 200) {
    cxx_test_suite_set_correct();
  }
}

void c_take_ref_rust_vec_copy(const rust::Vec<uint8_t> &v) {
  // The std::copy() will make sure rust::Vec<>::const_iterator satisfies the
  // requirements for std::iterator_traits.
  // https://en.cppreference.com/w/cpp/iterator/iterator_traits
  std::vector<uint8_t> stdv;
  std::copy(v.begin(), v.end(), std::back_inserter(stdv));
  uint8_t sum = std::accumulate(stdv.begin(), stdv.end(), 0);
  if (sum == 200) {
    cxx_test_suite_set_correct();
  }
}

/*
// https://github.com/dtolnay/cxx/issues/232
void c_take_callback(rust::Fn<size_t(rust::String)> callback) {
  callback("2020");
}
*/

void c_take_enum(Enum e) {
  if (e == Enum::AVal) {
    cxx_test_suite_set_correct();
  }
}

void c_try_return_void() {}

size_t c_try_return_primitive() { return 2020; }

size_t c_fail_return_primitive() { throw std::logic_error("logic error"); }

rust::Box<R> c_try_return_box() { return c_return_box(); }

const rust::String &c_try_return_ref(const rust::String &s) { return s; }

rust::Str c_try_return_str(rust::Str s) { return s; }

rust::Slice<uint8_t> c_try_return_sliceu8(rust::Slice<uint8_t> s) { return s; }

rust::String c_try_return_rust_string() { return c_return_rust_string(); }

std::unique_ptr<std::string> c_try_return_unique_ptr_string() {
  return c_return_unique_ptr_string();
}

rust::Vec<uint8_t> c_try_return_rust_vec() {
  throw std::runtime_error("unimplemented");
}

const rust::Vec<uint8_t> &c_try_return_ref_rust_vec(const C &c) {
  (void)c;
  throw std::runtime_error("unimplemented");
}

extern "C" C *cxx_test_suite_get_unique_ptr() noexcept {
  return std::unique_ptr<C>(new C{2020}).release();
}

extern "C" std::string *cxx_test_suite_get_unique_ptr_string() noexcept {
  return std::unique_ptr<std::string>(new std::string("2020")).release();
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
  ASSERT(cxx_test_suite_r_is_correct(&*r_return_box()));
  ASSERT(r_return_unique_ptr()->get() == 2020);
  ASSERT(r_return_ref(Shared{2020}) == 2020);
  ASSERT(std::string(r_return_str(Shared{2020})) == "2020");
  ASSERT(std::string(r_return_rust_string()) == "2020");
  ASSERT(*r_return_unique_ptr_string() == "2020");
  ASSERT(r_return_identity(2020) == 2020);
  ASSERT(r_return_sum(2020, 1) == 2021);
  ASSERT(r_return_enum(0) == Enum::AVal);
  ASSERT(r_return_enum(1) == Enum::BVal);
  ASSERT(r_return_enum(2021) == Enum::CVal);

  r_take_primitive(2020);
  r_take_shared(Shared{2020});
  r_take_unique_ptr(std::unique_ptr<C>(new C{2020}));
  r_take_ref_c(C{2020});
  r_take_str(rust::Str("2020"));
  r_take_sliceu8(rust::Slice<uint8_t>(
      reinterpret_cast<const uint8_t *>(SLICE_DATA), sizeof(SLICE_DATA)));
  r_take_rust_string(rust::String("2020"));
  r_take_unique_ptr_string(
      std::unique_ptr<std::string>(new std::string("2020")));
  r_take_enum(Enum::AVal);

  ASSERT(r_try_return_primitive() == 2020);
  try {
    r_fail_return_primitive();
    ASSERT(false);
  } catch (const rust::Error &e) {
    ASSERT(std::strcmp(e.what(), "rust error") == 0);
  }

  auto r2 = r_return_r2(2020);
  ASSERT(r2->get() == 2020);
  ASSERT(r2->set(2021) == 2021);
  ASSERT(r2->get() == 2021);
  ASSERT(r2->set(2020) == 2020);
  ASSERT(r2->get() == 2020);

  cxx_test_suite_set_correct();
  return nullptr;
}

} // namespace tests
