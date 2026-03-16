#include <iostream>
#include <cstdint>
#include "lib.h"

int main() {
    std::cout << "Testing shared library interaction..." << std::endl;

    std::int32_t magic = get_magic_number();
    std::cout << "magic number: " << magic << std::endl;
    if (magic != 1042) {
        std::cerr << "ERROR: expected 1042, got " << magic << std::endl;
        return 1;
    }

    std::int32_t product = multiply_values(3, 4);
    std::cout << "multiply result: " << product << std::endl;
    if (product != 24) {
        std::cerr << "ERROR: expected 24, got " << product << std::endl;
        return 1;
    }

    std::int32_t entry_result = library_entry_point();
    std::cout << "entry point result: " << entry_result << std::endl;
    if (entry_result != 1200) {
        std::cerr << "ERROR: expected 1200, got " << entry_result << std::endl;
        return 1;
    }

    std::cout << "All tests passed!" << std::endl;
    return 0;
}
