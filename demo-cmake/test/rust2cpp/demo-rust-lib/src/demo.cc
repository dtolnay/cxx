#include "demo.h"

#include <iostream>

#include "lib.rs.h"

namespace org {
namespace example {

ThingC::ThingC(std::string appname) : appname(appname) {}

ThingC::~ThingC() { std::cout << "done with ThingC" << std::endl; }

void print_name(const ThingC &thing) { print_str(thing.appname); }

} // namespace example
} // namespace org
