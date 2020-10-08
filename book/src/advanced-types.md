# Advanced types

## Raw pointers

At present, raw pointers are not natively supported in `cxx` interfaces.

However, as a workaround, you can transport them within a `UniquePtr`.

To transport a raw pointer from C++ to Rust, for instance:

```c++
Foo* existing_raw_pointer = // ...;
cxx_api(std::unique_ptr<Foo>(existing_raw_pointer))
```

```rust
#[cxx::bridge]
mod ffi {
    extern "C" {
        type Foo;
    }

    extern "Rust" {
        fn cxx_api(ptr: UniquePtr<Foo>);
    }
}

fn cxx_api(ptr: UniquePtr<Foo>) {
    let raw_ptr = ptr.into_raw();
    // Handle raw_ptr
}
```

and to transport a raw pointer from Rust to C++, you can use the `unsafe` function `UniquePtr::from_raw`.

## Working with bindgen types

If you've used `bindgen` to generate a swathe of existing C++ type definitions, you can refer to them from the `#[cxx::bridge]` mod using the [`ExternType`](https://docs.rs/cxx/0.4.7/cxx/trait.ExternType.html) trait. See also the mention of [higher level code generators](context.md) which may automate this in future, and a bit more on `ExternType` in the [chapter on C++ types](cpp-types.md).
