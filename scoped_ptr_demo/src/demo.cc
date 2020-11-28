#include "scoped-ptr-demo/include/demo.h"
#include "scoped-ptr-demo/src/main.rs.h"
#include <iostream>

Class::~Class() { std::cout << x << "::~Class " << std::endl; }
void Class::print() const { std::cout << x << "::print" << std::endl; }

rust_scoped_ptr<Class> getclass() noexcept {
  return {boost::scoped_ptr{new Class{9}}};
}
