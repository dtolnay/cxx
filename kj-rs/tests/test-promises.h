#pragma once

#include "kj-rs-demo/lib.rs.h"

#include <kj/async.h>

namespace kj_rs_demo {

kj::Promise<void> new_ready_promise_void();
kj::Promise<void> new_pending_promise_void();
kj::Promise<void> new_coroutine_promise_void();

kj::Promise<void> new_errored_promise_void();
kj::Promise<int32_t> new_ready_promise_i32(int32_t);
kj::Promise<Shared> new_ready_promise_shared_type();

}  // namespace kj_rs_demo
