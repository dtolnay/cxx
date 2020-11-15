#[cxx::bridge]
mod ffi {
    extern "C++" {
        type Opaque;
    }
}

fn assert_send<T: Send>() {}
fn assert_sync<T: Sync>() {}

fn main() {
    assert_send::<ffi::Opaque>();
    assert_sync::<ffi::Opaque>();
}
