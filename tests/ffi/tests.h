#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace tests {

struct R;
struct Shared;

class C {
public:
  C(size_t n);
  size_t get() const;
  size_t set(size_t n);

private:
  size_t n;
};

size_t c_return_primitive();
Shared c_return_shared();
rust::Box<R> c_return_box();
std::unique_ptr<C> c_return_unique_ptr();
const size_t &c_return_ref(const Shared &shared);
rust::Str c_return_str(const Shared &shared);
rust::Slice<uint8_t> c_return_sliceu8(const Shared &shared);
rust::String c_return_rust_string();
std::unique_ptr<std::string> c_return_unique_ptr_string();

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
void c_take_callback(rust::Fn<size_t(rust::String)> callback);

void c_try_return_void();
size_t c_try_return_primitive();
size_t c_fail_return_primitive();
rust::Box<R> c_try_return_box();
const rust::String &c_try_return_ref(const rust::String &);
rust::Str c_try_return_str(rust::Str);
rust::Slice<uint8_t> c_try_return_sliceu8(rust::Slice<uint8_t>);
rust::String c_try_return_rust_string();
std::unique_ptr<std::string> c_try_return_unique_ptr_string();

} // namespace tests
