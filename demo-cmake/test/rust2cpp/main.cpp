#include "demo.h"

int main(int argc, char **argv) {
    org::example::print_name(org::example::ThingC(argc < 2 ? "C++" : argv[1]));
    return 0;
}
