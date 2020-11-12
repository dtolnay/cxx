#include <iostream>
#include <mmscenegraph.h>

namespace mmscenegraph {

ThingC::ThingC(std::string appname) : appname(std::move(appname)) {}

ThingC::~ThingC() { std::cout << "done with ThingC" << std::endl; }

std::unique_ptr<ThingC> make_demo(rust::Str appname) {
    return std::unique_ptr<ThingC>(new ThingC(std::string(appname)));
}

const std::string &get_name(const ThingC &thing) { return thing.appname; }

void do_thing(SharedThing state) { print_r(*state.y); }

} // namespace mmscenegraph
