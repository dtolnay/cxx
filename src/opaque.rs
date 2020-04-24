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

const_assert_eq!(0, mem::size_of::<Opaque>());
const_assert_eq!(1, mem::align_of::<Opaque>());
