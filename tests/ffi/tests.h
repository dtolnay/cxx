#pragma once
#include "cxxbridge/cxxbridge.h"
#include <memory>
#include <string>

namespace tests {

struct R;
struct Shared;

class C {
public:
  C(size_t n);
  size_t get() const;

private:
  size_t n;
};

size_t c_return_primitive();
Shared c_return_shared();
rust::box<R> c_return_box();
std::unique_ptr<C> c_return_unique_ptr();
const size_t &c_return_ref(const Shared &shared);
rust::str c_return_str(const Shared &shared);
rust::string c_return_rust_string();
std::unique_ptr<std::string> c_return_unique_ptr_string();

void c_take_primitive(size_t n);
void c_take_shared(Shared shared);
void c_take_box(rust::box<R> r);
void c_take_unique_ptr(std::unique_ptr<C> c);
void c_take_ref_r(const R &r);
void c_take_ref_c(const C &c);
void c_take_str(rust::str s);
void c_take_rust_string(rust::string s);
void c_take_unique_ptr_string(std::unique_ptr<std::string> s);

} // namespace tests
