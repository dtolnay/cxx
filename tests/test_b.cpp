#include <iostream>
#include <string>
#include "mmscenegraph.h"

namespace mmsg = mmscenegraph;

int test_b() {
    auto app_name = std::string("my awesome demo");
    auto x = mmsg::make_demo(app_name);
    auto name = mmsg::get_name(*x);
    std::cout << name << std::endl;
    return 0;
}
