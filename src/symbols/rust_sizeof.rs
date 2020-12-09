use crate::rust_string::RustString;
use alloc::vec::Vec;
use core::mem;
use std::os::raw::c_char;

macro_rules! rust_stride_shims {
    ($segment:expr, $ty:ty) => {
        const_assert_eq!(mem::size_of::<[usize; 3]>(), mem::size_of::<Vec<$ty>>());
        const_assert_eq!(mem::align_of::<usize>(), mem::align_of::<Vec<$ty>>());

        const _: () = {
            attr! {
                #[export_name = concat!("cxxbridge1$rust_stride$", $segment)]
                unsafe extern "C" fn __stride() -> usize {
                    mem::size_of::<$ty>()
                }
            }
        };
    };
}

macro_rules! rust_stride_shims_for_primitive {
    ($ty:ident) => {
        rust_stride_shims!(stringify!($ty), $ty);
    };
}

rust_stride_shims_for_primitive!(bool);
rust_stride_shims_for_primitive!(u8);
rust_stride_shims_for_primitive!(u16);
rust_stride_shims_for_primitive!(u32);
rust_stride_shims_for_primitive!(u64);
rust_stride_shims_for_primitive!(i8);
rust_stride_shims_for_primitive!(i16);
rust_stride_shims_for_primitive!(i32);
rust_stride_shims_for_primitive!(i64);
rust_stride_shims_for_primitive!(f32);
rust_stride_shims_for_primitive!(f64);

rust_stride_shims!("char", c_char);
rust_stride_shims!("string", RustString);
