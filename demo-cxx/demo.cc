#include "demo-cxx/demo.h"
#include "demo-rs/src/main.rs.h"
#include <iostream>

namespace org {
namespace example {

ThingC::ThingC(std::string appname) : appname(std::move(appname)) {}

ThingC::~ThingC() { std::cout << "done with ThingC" << std::endl; }

const std::string &ThingC::get_name() const {
  std::cout << "I'm a C++ method!" << std::endl;
  return this->appname;
}

std::unique_ptr<ThingC> make_demo(rust::Str appname) {
  return std::unique_ptr<ThingC>(new ThingC(std::string(appname)));
}

void do_thing(SharedThing state) {
  print_r(*state.y);
  state.y->print();
}

} // namespace example
} // namespace org
