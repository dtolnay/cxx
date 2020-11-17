{{#title rust::Vec<T> — Rust ♡ C++}}
# rust::Vec\<T\>

### Public API:

```cpp,hidelines
// rust/cxx.h
#
# #include <iterator>
# #include <type_traits>
#
# namespace rust {

template <typename T>
class Vec final {
public:
  using value_type = T;

  Vec() noexcept;
  Vec(Vec &&) noexcept;
  ~Vec() noexcept;

  Vec &operator=(Vec &&) noexcept;

  size_t size() const noexcept;
  bool empty() const noexcept;
  const T *data() const noexcept;
  T *data() noexcept;

  const T &operator[](size_t n) const noexcept;
  const T &at(size_t n) const;

  const T &front() const;
  const T &back() const;

  void reserve(size_t new_cap);
  void push_back(const T &value);
  void push_back(T &&value);
  template <typename... Args>
  void emplace_back(Args &&... args);

  class const_iterator final {
  public:
    using difference_type = ptrdiff_t;
    using value_type = typename std::add_const<T>::type;
    using pointer =
        typename std::add_pointer<typename std::add_const<T>::type>::type;
    using reference = typename std::add_lvalue_reference<
        typename std::add_const<T>::type>::type;
    using iterator_category = std::forward_iterator_tag;

    const T &operator*() const noexcept;
    const T *operator->() const noexcept;
    const_iterator &operator++() noexcept;
    const_iterator operator++(int) noexcept;
    bool operator==(const const_iterator &) const noexcept;
    bool operator!=(const const_iterator &) const noexcept;
  };

  const_iterator begin() const noexcept;
  const_iterator end() const noexcept;
};
#
# } // namespace rust
```

### Restrictions:

Vec\<T\> does not support T being an opaque C++ type. You should use
CxxVector\<T\> (C++ std::vector\<T\>) instead for collections of opaque C++
types on the language boundary.

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

        fn f(elements: Vec<Shared>);
    }
}

fn main() {
    let shared = |v| ffi::Shared { v };
    let elements = vec![shared(3), shared(2), shared(1)];
    ffi::f(elements);
}
```

```cpp
// include/example.h

#pragma once
#include "example/src/main.rs.h"
#include "rust/cxx.h"

void f(rust::Vec<Shared> elements);
```

```cpp
// src/example.cc

#include "example/include/example.h"
#include <algorithm>
#include <cassert>
#include <iostream>
#include <iterator>
#include <vector>

void f(rust::Vec<Shared> v) {
  for (auto shared : v) {
    std::cout << shared.v << std::endl;
  }

  // Copy the elements to a C++ std::vector using STL algorithm.
  std::vector<Shared> stdv;
  std::copy(v.begin(), v.end(), std::back_inserter(stdv));
  assert(v.size() == stdv.size());
}
```
