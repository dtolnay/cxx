#include <kj/async-io.h>
#include <kj/async.h>
#include <kj/common.h>
#include <kj/test.h>

#include <algorithm>
#include <cstring>

#include "kj-rs-demo/lib.rs.h"

class ZeroInputStream : public kj::AsyncInputStream {
public:
  ZeroInputStream(size_t len) : rem(len) {}
  virtual ~ZeroInputStream() = default;

  kj::Promise<size_t> read(void *buffer, size_t minBytes,
                           size_t maxBytes) override {
    auto n = std::min(maxBytes, rem);
    memset(buffer, 0, n);
    rem -= n;
    return n;
  }

  kj::Promise<size_t> tryRead(void *buffer, size_t minBytes,
                              size_t maxBytes) override {
    return read(buffer, minBytes, maxBytes);
  }

private:
  size_t rem;
};

template<typename Impl>
class RustAsyncInputStream final: public kj::AsyncInputStream {
public:
  RustAsyncInputStream(::rust::Box<Impl> impl) : impl(kj::mv(impl)) {}
  virtual ~RustAsyncInputStream() = default;

  kj::Promise<size_t> tryRead(void *buffer, size_t minBytes, size_t maxBytes) override {
    auto slice = ::rust::Slice(static_cast<uint8_t*>(buffer), maxBytes);
    return impl->try_read(slice, maxBytes);
  }

private:
  ::rust::Box<Impl> impl;
};

template<size_t bufferSize>
size_t readFullStream(kj::AsyncInputStream& stream, size_t expectedLen) {
  kj::EventLoop loop;
  kj::Maybe<kj::Exception> maybeException;
  kj::WaitScope waitScope(loop);

  return [&]() -> kj::Promise<size_t> {
    auto buffer = kj::heapArray<kj::byte>(bufferSize);
    size_t len = 0;

    while (true) {
      size_t n = co_await stream.tryRead(buffer.begin(), bufferSize, bufferSize);
      if (n == 0) {
        break;
      }
      len += n;
    }
    co_return len;
  }().wait(waitScope);
}

KJ_TEST("C++ ZeroInputStream") {
  constexpr auto size = 1024;
  ZeroInputStream stream(size);
  KJ_ASSERT(readFullStream<127>(stream, size) == size);
}

KJ_TEST("Rust ZeroInputStream") {
  constexpr auto size = 1024;
  RustAsyncInputStream stream(kj_rs_demo::new_zero_stream(size));
  KJ_ASSERT(readFullStream<127>(stream, size) == size);
}

#ifdef NDEBUG
// ~1sec
constexpr auto benchmarkSize = 1024 * 1024 * 1024 * 10l;
#else
constexpr auto benchmarkSize = 1024 * 1024;
#endif

KJ_TEST("Benchmark C++ ZeroInputStream") {
  ZeroInputStream stream(benchmarkSize);
  KJ_ASSERT(readFullStream<1024 + 1>(stream, benchmarkSize) == benchmarkSize);
}

KJ_TEST("Benchmark Rust ZeroInputStream") {
  RustAsyncInputStream stream(kj_rs_demo::new_zero_stream(benchmarkSize));
  KJ_ASSERT(readFullStream<1024 + 1>(stream, benchmarkSize) == benchmarkSize);
}
