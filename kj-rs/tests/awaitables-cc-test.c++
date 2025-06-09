// TODO(now): Make this a library, drive test from Rust.
// TODO(now): Move as many cases as possible into kj-rs.

#include "kj-rs-demo/test-promises.h"
#include "kj-rs/awaiter.h"
#include "kj-rs/future.h"
#include "kj-rs/waker.h"

#include <kj/test.h>

namespace kj_rs_demo {
namespace {

KJ_TEST("polling pending future") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  kj::Promise<void> promise = new_pending_future_void();
  KJ_EXPECT(!promise.poll(waitScope));
}

KJ_TEST("C++ KJ coroutine can co_await rust ready void future") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_ready_future_void(); }().wait(waitScope);
}

KJ_TEST("C++ KJ coroutines can co_await Rust Futures") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> {
    co_await new_ready_future_void();
    co_await new_waking_future_void(CloningAction::None, WakingAction::WakeByRefSameThread);
  }().wait(waitScope);
}

KJ_TEST("c++ can receive synchronous wakes during poll()") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  struct Actions {
    CloningAction cloningAction;
    WakingAction wakingAction;
  };

  for (auto testCase: std::initializer_list<Actions>{
         {CloningAction::None, WakingAction::WakeByRefSameThread},
         {CloningAction::None, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneSameThread, WakingAction::WakeByRefSameThread},
         {CloningAction::CloneSameThread, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeByRefSameThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeByRefBackgroundThread},
         {CloningAction::CloneSameThread, WakingAction::WakeSameThread},
         {CloningAction::CloneSameThread, WakingAction::WakeBackgroundThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeSameThread},
         {CloningAction::CloneBackgroundThread, WakingAction::WakeBackgroundThread},
         {CloningAction::WakeByRefThenCloneSameThread, WakingAction::WakeSameThread},
       }) {
    auto waking = new_waking_future_void(testCase.cloningAction, testCase.wakingAction);
    KJ_EXPECT(waking.poll(waitScope));
    waking.wait(waitScope);
  }
}

KJ_TEST("RustPromiseAwaiter: Rust can .await KJ promises under a co_await") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_layered_ready_future_void(); }().wait(waitScope);
}

KJ_TEST("RustPromiseAwaiter: Rust can poll() multiple promises under a single co_await") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_naive_select_future_void(); }().wait(waitScope);
}

// TODO(now): Similar to "Rust can poll() multiple promises ...", but poll() until all are ready.

// TODO(now): Test polling a Promise from Rust with multiple LazyArcWakers.
//   Need a function which:
//   - Creates an OwnPromiseNode which is fulfilled manually.
//   - Wraps OwnPromiseNode::into_future() in BoxFuture.
//   - Passes the BoxFuture to a new KJ coroutine.
//   - The KJ coroutine passes the BoxFuture to a Rust function returning NaughtyFuture.
//   - The coroutine co_awaits the NaughtyFuture.
//   - The NaughtyFuture polls the BoxFuture once and returns Ready(BoxFuture).
//   - The coroutine co_returns the BoxFuture to the local function here.
//   - The BoxFuture has now outlived the coroutine which polled it last.
//   - Fulfill the OwnPromiseNode. Should not segfault.
//   - Pass the OwnPromiseNode to a new Rust Future somehow, .await it.

KJ_TEST("RustPromiseAwaiter: Rust can poll() KJ promises with non-KJ Wakers") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_wrapped_waker_future_void(); }().wait(waitScope);
}

KJ_TEST("co_awaiting a fallible future from C++ can throw") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> {
    kj::Maybe<kj::Exception> maybeException;
    try {
      co_await new_errored_future_void();
    } catch (...) {
      maybeException = kj::getCaughtExceptionAsKj();
    }
    auto& exception = KJ_ASSERT_NONNULL(maybeException, "should have thrown");
    KJ_EXPECT(exception.getDescription() == "test error");
  }().wait(waitScope);
}

KJ_TEST(".awaiting a Promise<T> from Rust can produce an Err Result") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_error_handling_future_void_infallible(); }().wait(
           waitScope);
}

KJ_TEST("Rust can await Promise<int32_t>") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { co_await new_awaiting_future_i32(); }().wait(waitScope);
}

KJ_TEST("C++ can await BoxFuture<i32>") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  []() -> kj::Promise<void> { KJ_EXPECT(co_await new_ready_future_i32(123) == 123); }().wait(
           waitScope);
}

KJ_TEST("C++ can receive asynchronous wakes after poll()") {
  kj::EventLoop loop;
  kj::WaitScope waitScope(loop);

  auto promise = new_threaded_delay_future_void();
  // It's not ready yet.
  KJ_EXPECT(!promise.poll(waitScope));
  // But later it is.
  promise.wait(waitScope);
}

// TODO(now): More test cases.
//   - Standalone ArcWaker tests. Ensure Rust calls ArcWaker destructor when we expect.
//   - Ensure Rust calls PromiseNode destructor from LazyRustPromiseAwaiter.
//   - Throwing an exception from PromiseNode functions, including destructor.

}  // namespace
}  // namespace kj_rs_demo