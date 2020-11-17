{{#title rust::Slice<T> — Rust ♡ C++}}
# rust::Slice\<T\>

### Public API:

```cpp,hidelines
// rust/cxx.h
#
# namespace rust {

template <typename T>
class Slice final {
public:
  Slice() noexcept;
  Slice(const Slice<T> &) noexcept;
  Slice(T *, size_t count) noexcept;

  Slice &operator=(const Slice<T> &) noexcept;

  T *data() const noexcept;
  size_t size() const noexcept;
  size_t length() const noexcept;
};
#
# } // namespace rust
```

### Restrictions:

Only &amp;\[u8\] i.e. rust::Slice\<const uint8\_t\> is currently implemented.
Support for arbitrary &amp;\[T\] is coming.

Allowed as function argument or return value. Not supported in shared structs.
&amp;mut \[T\] is not supported yet.

## Example

This example is a C++ program that constructs a slice containing JSON data (by
reading from stdin, but it could be from anywhere), then calls into Rust to
pretty-print that JSON data into a std::string via the [serde_json] and
[serde_transcode] crates.

[serde_json]: https://github.com/serde-rs/json
[serde_transcode]: https://github.com/sfackler/serde-transcode

```rust,noplayground
// src/main.rs

#![no_main] // main defined in C++ by main.cc

use cxx::CxxString;
use std::io::{self, Write};
use std::pin::Pin;

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        fn prettify_json(input: &[u8], output: Pin<&mut CxxString>) -> Result<()>;
    }
}

struct WriteToCxxString<'a>(Pin<&'a mut CxxString>);

impl<'a> Write for WriteToCxxString<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.as_mut().push_bytes(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn prettify_json(input: &[u8], output: Pin<&mut CxxString>) -> serde_json::Result<()> {
    let writer = WriteToCxxString(output);
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    let mut serializer = serde_json::Serializer::pretty(writer);
    serde_transcode::transcode(&mut deserializer, &mut serializer)
}
```

```cpp
// src/main.cc

#include "example/src/main.rs.h"
#include <iostream>
#include <iterator>
#include <string>
#include <vector>

int main() {
  // Read json from stdin.
  std::istreambuf_iterator<char> begin{std::cin}, end;
  std::vector<unsigned char> input{begin, end};
  rust::Slice<const uint8_t> slice{input.data(), input.size()};

  // Prettify using serde_json and serde_transcode.
  std::string output;
  prettify_json(slice, output);

  // Write to stdout.
  std::cout << output << std::endl;
}
```

Testing the example:

```console
$  echo '{"fearless":"concurrency"}' | cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.02s
     Running `target/debug/example`
{
  "fearless": "concurrency"
}
```
