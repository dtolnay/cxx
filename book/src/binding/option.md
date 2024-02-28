{{#title rust::Option<T> — Rust ♡ C++}}
# rust::Option\<T\>

### Public API:

```cpp,hidelines=...
// rust/cxx.h
...
...namespace rust {

template <typename T>
class Option final {
public:
  Option() noexcept;
  Option(Option&&) noexcept;
  Option(T&&) noexcept;
  ~Option() noexcept;

  const T *operator->() const;
  const T &operator*() const;
  T *operator->();
  T &operator*();

  Option<T>& operator=(Option&&) noexcept;

  bool has_value() const noexcept;
  T& value() noexcept;
  void reset();
  void set(T&&) noexcept;
};
...} // namespace rust
```

### Restrictions:

Option<T> only supports pointer-sized references and Box-es; that is, no
fat pointers like &str (though &String is supported) or Box<[u8]>. On the
C++ side, Option<&T> becomes rust::Option<const T*> (and similar for
mutable references), but the pointer is guaranteed to be non-null if the
Option has a value. Also, you can only pass Options themselves by value.
&Option<T> is not allowed.

## Example

```rust,noplayground
// src/main.rs

#[cxx::bridge]
mod ffi {
    struct Shared {
        v: u32,
    }

    unsafe extern "C++" {
        include!("example/include/example.h");

        fn f(elements: Option<&Shared>);
    }
}

fn main() {
    let shared = Shared { v: 3 };
    ffi::f(Some(&shared));
    ffi::f(None);
}
```

```cpp
// include/example.h

#pragma once
#include "example/src/main.rs.h"
#include "rust/cxx.h"

void f(rust::Option<const Shared*>);
```

```cpp
// src/example.cc

#include "example/include/example.h"

void f(rust::Option<const Shared*> o) {
  if (o.has_value()) {
    // Pointer is guaranteed to be non-null
    std::cout << shared.value()->v << std::endl;
  }
}
```
