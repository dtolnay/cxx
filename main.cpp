#include <iostream>
#include "rust_part.hpp"

int main()
{
    auto thing = rust_part::make_shared_thing();
    rust_part::print_shared_thing(thing);
}
