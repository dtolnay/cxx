#include "tests.h"

namespace kj_rs {

kj::Promise<void> c_async_void_fn() { return kj::READY_NOW; }

kj::Promise<int64_t> c_async_int_fn() { return 42; }

kj::Promise<Shared> c_async_struct_fn() { return Shared{42}; }

} // namespace kj_rs
