use std::mem;

// . size = 0
// . align = 1
// . ffi-safe
// . !Send
// . !Sync
#[repr(C, packed)]
pub struct Opaque {
    _private: [*const u8; 0],
}

fn _assert() {
    let _: [(); 0] = [(); mem::size_of::<Opaque>()];
    let _: [(); 1] = [(); mem::align_of::<Opaque>()];
}
