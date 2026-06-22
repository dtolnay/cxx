#include "exe_functions.h"

// Implementation of exe callback functions
int exe_callback(int value) {
    // doubles the value
    return value * 2;
}

int exe_get_constant() {
    return 1000;
}
