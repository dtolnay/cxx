#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace A {
struct AShared;
enum class AEnum : uint16_t;
namespace B {
struct ABShared;
enum class ABEnum : uint16_t;
} // namespace B
} // namespace A

namespace F {
struct F {
  uint64_t f;
  std::string f_str;
};
} // namespace F

namespace G {
struct G {
  uint64_t g;
};
} // namespace G

namespace H {
class H {
public:
  std::string h;
};
} // namespace H

namespace tests {

struct R;
struct Shared;
struct SharedString;
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
  rust::String cOverloadedMethod(int32_t x) const;
  rust::String cOverloadedMethod(rust::Str x) const;

private:
  size_t n;
  std::vector<uint8_t> v;
};

struct D {
  uint64_t d;
};

struct E {
  uint64_t e;
  std::string e_str;
};

enum COwnedEnum {
  CVal1,
  CVal2,
};

size_t c_return_primitive();
Shared c_return_shared();
::A::AShared c_return_ns_shared();
::A::B::ABShared c_return_nested_ns_shared();
rust::Box<R> c_return_box();
std::unique_ptr<C> c_return_unique_ptr();
std::unique_ptr<::H::H> c_return_ns_unique_ptr();
const size_t &c_return_ref(const Shared &shared);
const size_t &c_return_ns_ref(const ::A::AShared &shared);
const size_t &c_return_nested_ns_ref(const ::A::B::ABShared &shared);
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
::A::AEnum c_return_ns_enum(uint16_t n);
::A::B::ABEnum c_return_nested_ns_enum(uint16_t n);

void c_take_primitive(size_t n);
void c_take_shared(Shared shared);
void c_take_ns_shared(::A::AShared shared);
void c_take_nested_ns_shared(::A::B::ABShared shared);
void c_take_box(rust::Box<R> r);
void c_take_unique_ptr(std::unique_ptr<C> c);
void c_take_ref_r(const R &r);
void c_take_ref_c(const C &c);
void c_take_ref_ns_c(const ::H::H &h);
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
void c_take_rust_vec_ns_shared(rust::Vec<::A::AShared> v);
void c_take_rust_vec_nested_ns_shared(rust::Vec<::A::B::ABShared> v);
void c_take_rust_vec_string(rust::Vec<rust::String> v);
void c_take_rust_vec_shared_index(rust::Vec<Shared> v);
void c_take_rust_vec_shared_push(rust::Vec<Shared> v);
void c_take_rust_vec_shared_forward_iterator(rust::Vec<Shared> v);
void c_take_ref_rust_vec(const rust::Vec<uint8_t> &v);
void c_take_ref_rust_vec_string(const rust::Vec<rust::String> &v);
void c_take_ref_rust_vec_index(const rust::Vec<uint8_t> &v);
void c_take_ref_rust_vec_copy(const rust::Vec<uint8_t> &v);
const SharedString &c_take_ref_shared_string(const SharedString &s);
void c_take_callback(rust::Fn<size_t(rust::String)> callback);
void c_take_enum(Enum e);
void c_take_ns_enum(::A::AEnum e);
void c_take_nested_ns_enum(::A::B::ABEnum e);

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

void c_take_trivial_ptr(std::unique_ptr<D> d);
void c_take_trivial_ref(const D &d);
void c_take_trivial(D d);

void c_take_trivial_ns_ptr(std::unique_ptr<::G::G> g);
void c_take_trivial_ns_ref(const ::G::G &g);
void c_take_trivial_ns(::G::G g);
void c_take_opaque_ptr(std::unique_ptr<E> e);
void c_take_opaque_ns_ptr(std::unique_ptr<::F::F> f);
void c_take_opaque_ref(const E &e);
void c_take_opaque_ns_ref(const ::F::F &f);
std::unique_ptr<D> c_return_trivial_ptr();
D c_return_trivial();
std::unique_ptr<::G::G> c_return_trivial_ns_ptr();
::G::G c_return_trivial_ns();
std::unique_ptr<E> c_return_opaque_ptr();
std::unique_ptr<::F::F> c_return_ns_opaque_ptr();

rust::String cOverloadedFunction(int32_t x);
rust::String cOverloadedFunction(rust::Str x);

} // namespace tests

namespace other {
void ns_c_take_trivial(::tests::D d);
::tests::D ns_c_return_trivial();
void ns_c_take_ns_shared(::A::AShared shared);
} // namespace other

namespace I {
class I {
private:
  uint32_t a;

public:
  I() : a(1000) {}
  uint32_t get() const;
};

std::unique_ptr<I> ns_c_return_unique_ptr_ns();
} // namespace I
