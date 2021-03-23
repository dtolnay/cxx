{{#title *mut T, *const T — Rust ♡ C++}}
# *mut T, *const T - Raw Pointers

cxx supports tranfer of raw pointers across the FFI boundary.

Generally, it's better to use the bindings for [std::unique_ptr\<T\>](uniqueptr.md)
or references. You should resort to using raw pointers only where lifetimes
are too complicated to model with standard cxx facilities.

As is normal with raw pointers in Rust, you'll need to use `unsafe` when
working with them. In particular, to pass any raw pointer into a cxx
bridge function you will need to declare the function `unsafe`, even if
the overall `extern "C++"` section is already marked as `unsafe`. By calling
such a function, you're committing that you - the human - know enough about
the lifetimes of those objects that the compiler doesn't need to do checks.

On the other hand, C++ functions can freely return raw pointers to
Rust without `unsafe`, but actually using those raw pointers in any way is likely
to require an `unsafe` keyword.

## Example

```rust,noplayground
// src/main.rs

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {

        include!("tests/ffi/tests.h");
        //include!("example/include/container.h");

        type ComplexHierarchicContainer;

        fn new_hierarchic_container() -> UniquePtr<ComplexHierarchicContainer>;
        fn add_value(self: Pin<&mut ComplexHierarchicContainer>, key: &CxxString, value: &CxxString);
        fn add_child(self: Pin<&mut ComplexHierarchicContainer>, key: &CxxString) -> *mut ComplexHierarchicContainer;
        // ...
    }
}

fn main() {
    let mut container = ffi17::new_hierarchic_container();
    cxx::let_cxx_string!(key = "a");
    let mut subcontainer = container.pin_mut().add_child(&key);
    let mut subcontainer = unsafe { std::pin::Pin::new_unchecked(subcontainer.as_mut().unwrap()) };
    cxx::let_cxx_string!(key2 = "b");
    cxx::let_cxx_string!(value = "c");
    subcontainer.add_value(&key2, &value);
    // ...
}
```

```cpp
// include/container.h

#pragma once

class ComplexHierarchicContainer {
public:
    ComplexHierarchicContainer();
    void add_value(const std::string& key, const std::string& value);
    ComplexHierarchicContainer* add_child(const std::string& key);
    // ...
};

std::unique_ptr<ComplexHierarchicContainer> new_hierarchic_container();
```
