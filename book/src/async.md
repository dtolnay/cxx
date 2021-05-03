{{#title Async functions — Rust ♡ C++}}
# Async functions

Direct FFI of async functions is absolutely in scope for CXX (on C++20 and up)
but is not implemented yet in the current release. We are aiming for an
implementation that is as easy as:

```rust,noplayground
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        async fn doThing(arg: Arg) -> Ret;
    }
}
```

```cpp,hidelines
rust::Future<Ret> doThing(Arg arg) {
  auto v1 = co_await f();
  auto v2 = co_await g(arg);
  co_return v1 + v2;
}
```

## Workaround

For now the recommended approach is to handle the return codepath over a oneshot
channel (such as [`futures::channel::oneshot`]) represented in an opaque Rust
type on the FFI.

[`futures::channel::oneshot`]: https://docs.rs/futures/0.3.8/futures/channel/oneshot/index.html

```rust,noplayground
// bridge.rs

use futures::channel::oneshot;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        type DoThingContext;
    }

    unsafe extern "C++" {
        include!("path/to/bridge_shim.h");

        fn shim_doThing(
            arg: Arg,
            done: fn(Box<DoThingContext>, ret: Ret),
            ctx: Box<DoThingContext>,
        );
    }
}

struct DoThingContext(oneshot::Sender<Ret>);

pub async fn do_thing(arg: Arg) -> Ret {
    let (tx, rx) = oneshot::channel();
    let context = Box::new(DoThingContext(tx));

    ffi::shim_doThing(
        arg,
        |context, ret| { let _ = context.0.send(ret); },
        context,
    );

    rx.await.unwrap()
}
```

```cpp
// bridge_shim.cc

#include "path/to/bridge.rs.h"
#include "rust/cxx.h"

void shim_doThing(
    Arg arg,
    rust::Fn<void(rust::Box<DoThingContext> ctx, Ret ret)> done,
    rust::Box<DoThingContext> ctx) noexcept {
  doThing(arg)
      .then([done, ctx(std::move(ctx))](auto &&res) mutable {
        (*done)(std::move(ctx), std::move(res));
      });
}
```

### Streams

The async approach can be extended to support streaming data as well using a channel such as [`futures::channel::mpsc::unbounded`] instead of oneshot and passing `DoThingContext` to the closure as a reference:

[`futures::channel::mpsc::unbounded`]: https://docs.rs/futures/0.3.8/futures/channel/mpsc/fn.unbounded.html

```cpp
// bridge_shim.cc

#include "path/to/bridge.rs.h"
#include "rust/cxx.h"

void shim_doThing(
    Arg arg,
    rust::Fn<void(const DoThingContext &ctx, Ret ret)> done,
    rust::Box<DoThingContext> ctx) noexcept {
        done(*ctx, std::move(res));
        done(*ctx, std::move(res));
        done(*ctx, std::move(res));
}
```
