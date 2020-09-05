#include "demo/include/demo.h"
#include "demo/src/main.rs.h"
#include <iostream>

namespace org {
namespace example {

ThingC::ThingC(std::string appname) : appname(std::move(appname)) {}

ThingC::~ThingC() { std::cout << "done with ThingC" << std::endl; }

std::unique_ptr<ThingC> make_demo(rust::Str appname) {
  return std::make_unique<ThingC>(std::string(appname));
}

const std::string &get_name(const ThingC &thing) { return thing.appname; }

void do_thing(SharedThing state) { print_r(*state.y); }

void throws_strange() {
  throw 99;
}

} // namespace example
} // namespace org
