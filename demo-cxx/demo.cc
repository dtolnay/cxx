#include "demo-cxx/demo.h"
#include "demo-rs/src/main.rs.h"
#include <iostream>

namespace org {
namespace example {

ThingC::ThingC(std::string appname) : appname(std::move(appname)) {}

ThingC::~ThingC() { std::cout << "done with ThingC" << std::endl; }

void ThingC::camelCaseMethod() const { std::cout << "camelCaseMethod" << std::endl; }
void ThingC::overloadedMethod(int x) const { std::cout << "overloadedMethod: int x = " << x << std::endl; }
void ThingC::overloadedMethod(float x) const { std::cout << "overloadedMethod: float x = " << x << std::endl; }

std::unique_ptr<ThingC> make_demo(rust::Str appname) {
  return std::make_unique<ThingC>(std::string(appname));
}

void camelCaseFunction() { std::cout << "camelCaseFunction" << std::endl; }
void overloadedFunction(int x) { std::cout << "overloadedFunction: int x = " << x << std::endl; }
void overloadedFunction(float x) { std::cout << "overloadedFunction: float x = " << x << std::endl; }

const std::string &get_name(const ThingC &thing) { return thing.appname; }

void do_thing(SharedThing state) { print_r(*state.y); }

} // namespace example
} // namespace org
