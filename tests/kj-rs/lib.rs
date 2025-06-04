#![allow(dead_code)]
#![allow(clippy::unused_async)]

type Result<T> = std::io::Result<T>;

#[cxx::bridge(namespace = "kj_rs")]
pub mod ffi {
    struct Shared {
        i: i64,
    }

    unsafe extern "C++" {
        include!("tests/kj-rs/tests.h");

        async fn c_async_void_fn();
        async fn c_async_int_fn() -> i64;
        async fn c_async_struct_fn() -> Shared;
    }

    extern "Rust" {
        async fn rust_async_void_fn();
        async fn rust_async_int_fn() -> i64;

        async fn rust_async_void_result_fn() -> Result<()>;
        async fn rust_async_int_result_fn() -> Result<i64>;
    }
}

async fn rust_async_void_fn() {
}

async fn rust_async_int_fn() -> i64 {
    42
}

async fn rust_async_void_result_fn() -> Result<()> {
    Ok(())
}
 
async fn rust_async_int_result_fn() -> Result<i64> {
    Ok(42)
}

#[cfg(test)]
mod tests {
    use crate::ffi;

    // let kj-rs verify the behavior, just check compilation
    #[allow(clippy::let_underscore_future)]
    #[test]
    fn compilation() {
        let _ =  ffi::c_async_void_fn();
        let _ =  ffi::c_async_int_fn();
        let _ =  ffi::c_async_struct_fn();
    }
}
