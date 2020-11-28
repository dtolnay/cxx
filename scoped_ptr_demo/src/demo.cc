#include "scoped-ptr-demo/include/demo.h"
#include "scoped-ptr-demo/src/main.rs.h"
#include <iostream>

Class::~Class() { std::cout << x << "::~Class " << std::endl; }
void Class::print() const { std::cout << x << "::print" << std::endl; }

void run() {
  recv(boost::scoped_ptr{new Class{9}}, boost::scoped_ptr{new Class{1}});
}
