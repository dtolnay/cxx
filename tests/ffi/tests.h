#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace tests {

struct R;
struct Shared;
enum class Enum : uint16_t;

class C {
public:
  C(size_t n);
  size_t get() const;
  size_t set(size_t n);
  size_t get2() const;
  size_t set2(size_t n);
  size_t set_succeed(size_t n);
  size_t get_fail();
  const std::vector<uint8_t> &get_v() const;
  std::vector<uint8_t> &get_v();

private:
  size_t n;
  std::vector<uint8_t> v;
};

enum COwnedEnum {
  CVal1,
  CVal2,
};

size_t c_return_primitive();
Shared c_return_shared();
rust::Box<R> c_return_box();
std::unique_ptr<C> c_return_unique_ptr();
const size_t &c_return_ref(const Shared &shared);
size_t &c_return_mut(Shared &shared);
rust::Str c_return_str(const Shared &shared);
rust::Slice<uint8_t> c_return_sliceu8(const Shared &shared);
rust::String c_return_rust_string();
std::unique_ptr<std::string> c_return_unique_ptr_string();
std::unique_ptr<std::vector<uint8_t>> c_return_unique_ptr_vector_u8();
std::unique_ptr<std::vector<double>> c_return_unique_ptr_vector_f64();
std::unique_ptr<std::vector<std::string>> c_return_unique_ptr_vector_string();
std::unique_ptr<std::vector<Shared>> c_return_unique_ptr_vector_shared();
std::unique_ptr<std::vector<C>> c_return_unique_ptr_vector_opaque();
const std::vector<uint8_t> &c_return_ref_vector(const C &c);
std::vector<uint8_t> &c_return_mut_vector(C &c);
rust::Vec<uint8_t> c_return_rust_vec();
const rust::Vec<uint8_t> &c_return_ref_rust_vec(const C &c);
rust::Vec<uint8_t> &c_return_mut_rust_vec(C &c);
rust::Vec<rust::String> c_return_rust_vec_string();
size_t c_return_identity(size_t n);
size_t c_return_sum(size_t n1, size_t n2);
Enum c_return_enum(uint16_t n);

void c_take_primitive(size_t n);
void c_take_shared(Shared shared);
void c_take_box(rust::Box<R> r);
void c_take_unique_ptr(std::unique_ptr<C> c);
void c_take_ref_r(const R &r);
void c_take_ref_c(const C &c);
void c_take_str(rust::Str s);
void c_take_sliceu8(rust::Slice<uint8_t> s);
void c_take_rust_string(rust::String s);
void c_take_unique_ptr_string(std::unique_ptr<std::string> s);
void c_take_unique_ptr_vector_u8(std::unique_ptr<std::vector<uint8_t>> v);
void c_take_unique_ptr_vector_f64(std::unique_ptr<std::vector<double>> v);
void c_take_unique_ptr_vector_string(
    std::unique_ptr<std::vector<std::string>> v);
void c_take_unique_ptr_vector_shared(std::unique_ptr<std::vector<Shared>> v);
void c_take_ref_vector(const std::vector<uint8_t> &v);
void c_take_rust_vec(rust::Vec<uint8_t> v);
void c_take_rust_vec_index(rust::Vec<uint8_t> v);
void c_take_rust_vec_shared(rust::Vec<Shared> v);
void c_take_rust_vec_string(rust::Vec<rust::String> v);
void c_take_rust_vec_shared_index(rust::Vec<Shared> v);
void c_take_rust_vec_shared_forward_iterator(rust::Vec<Shared> v);
void c_take_ref_rust_vec(const rust::Vec<uint8_t> &v);
void c_take_ref_rust_vec_string(const rust::Vec<rust::String> &v);
void c_take_ref_rust_vec_index(const rust::Vec<uint8_t> &v);
void c_take_ref_rust_vec_copy(const rust::Vec<uint8_t> &v);
/*
// https://github.com/dtolnay/cxx/issues/232
void c_take_callback(rust::Fn<size_t(rust::String)> callback);
*/
void c_take_enum(Enum e);

void c_try_return_void();
size_t c_try_return_primitive();
size_t c_fail_return_primitive();
rust::Box<R> c_try_return_box();
const rust::String &c_try_return_ref(const rust::String &);
rust::Str c_try_return_str(rust::Str);
rust::Slice<uint8_t> c_try_return_sliceu8(rust::Slice<uint8_t>);
rust::String c_try_return_rust_string();
std::unique_ptr<std::string> c_try_return_unique_ptr_string();
rust::Vec<uint8_t> c_try_return_rust_vec();
rust::Vec<rust::String> c_try_return_rust_vec_string();
const rust::Vec<uint8_t> &c_try_return_ref_rust_vec(const C &c);

} // namespace tests

namespace alias_tests {

// These aliases on the C++ side aren't under test, there's just no reason to
// duplicate these functions
using tests::c_return_unique_ptr;
using tests::c_take_unique_ptr;

struct DifferentC {
  size_t n;
};

std::unique_ptr<alias_tests::DifferentC> create_different_c();

} // namespace alias_tests

namespace alias2_tests {

using alias_tests::create_different_c;
using tests::c_return_unique_ptr;
using tests::c_take_unique_ptr;

} // namespace alias2_tests
