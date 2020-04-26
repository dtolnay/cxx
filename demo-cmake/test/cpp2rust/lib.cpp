#include <iostream>
#include <string_view>

extern "C" void cpp_function(char const *name) {
    std::string_view const name_sv = name;
    std::cout << "Hello, " << name_sv << "! I'm C++!\n";
}