use std::cell::Cell;

thread_local! {
    pub static DROPPED: Cell<bool> = Cell::new(false);
}

#[cxx::bridge(namespace = "tests")]
pub mod ffi {
    struct DropShared {
        foo: usize,
    }

    unsafe extern "C++" {
        include!("tests/ffi/tests.h");

        fn c_return_drop_shared() -> DropShared;
        fn c_take_drop_shared(shared: DropShared);
    }
}

impl Drop for ffi::DropShared {
    fn drop(&mut self) {
        DROPPED.with(|dropped| dropped.set(true));
    }
}
