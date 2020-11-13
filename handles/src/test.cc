#include "cxx-handles-demo/src/main.rs.h"

void test() {
  my_usage::ffi::create_process("name?", zx::ffi::Job{1});
}
