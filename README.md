# Workerd-cxx - C++/Rust interop for Cloudflare Workerd

**This project is a fork of an excellent [cxx](https://crates.io/crates/cxx) crate for
[workerd](https://github.com/cloudflare/workerd) ecosystem.**

See [README.md.orig](README.md.orig) and [upstream documentation](https://cxx.rs/)
for the original information about cxx crate and its basic features.

This README concerns itself with the differences between the `cxx` crate and the fork.

## Development

Workerd-cxx can be built and consumed only using bazel. See `justfile` for common commands.

## Differences/Features

### Safety: `kj::Exception` integration and panic handling

Generated bridge is fully compatible with KJ exception model:

- C++ code is assumed to always throw and returns `Result<T, ::cxx::KjException>`.
- Rust code returning any `Result<T, E>` will convert errors to `kj::Exception` using `Display`.
- Rust code using error type `::cxx::KjError` can fully control information in the thrown exception.
- `kj::CanceledException` causes panic with `::cxx::CanceledException` payload.
- panic with `::cxx::CanceledException` causes `kj::CanceledException` to be thrown.
- any other panic will result in `kj::Exception` to be thrown.

### KJ Smart Pointers Integration

The following smart pointers can be used in ffi layers:

- `kj::Own<T>` - corresponds to `kj_rs::KjOwn<T>`
- `kj::Rc<T>` - corresponds to `kj_rs::KjRc<T>`
- `kj::Arc<T>` - corresponds to `kj_rs::KjArc<T>`

### KJ Data Structures Integration

- `kj::Maybe<T>` - corresponds to `kj_rs::KjMaybe<T>`.

### KJ/Rust conversion layer

Comprehensive conversion layer is provided for many KJ types through [`convert.h`](kj-rs/convert.h).

### Async fn support

FFI layer fully supports `async fn` functions. The resulting bridge will:

- expose `kj::Promise<T>` as `impl Future<Output = T>` to Rust code.
- expose `Future<Output = T>` as `kj::Promise<T>` to C++ code.

The resulting bridge promises and futures can be driven only by KJ event loop.
You can still drive Rust native futures by other rust event loops like tokio when no ffi promises
are used.

### Rust type aliases support

We have merged in upstream PR that enables
[rust type aliases](https://github.com/dtolnay/cxx/pull/1181), which are important for reusing
common cxx definitions across crates.
