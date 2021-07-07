use crate::rust_option::RustOption;
use crate::rust_string::RustString;
use core::ptr;
use std::os::raw::c_char;

macro_rules! rust_option_shims {
    ($segment:expr, $ty:ty) => {
        const _: () = {
            attr! {
                #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$new")]
                unsafe extern "C" fn __new(this: *mut RustOption<$ty>) {
                    ptr::write(this, RustOption::new());
                }
            }
            attr! {
                #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$drop")]
                unsafe extern "C" fn __drop(this: *mut RustOption<$ty>) {
                    ptr::drop_in_place(this);
                }
            }
        };
    };
}

macro_rules! rust_option_shims_for_primitive {
    ($ty:ident) => {
        rust_option_shims!(stringify!($ty), $ty);
    };
}

rust_option_shims_for_primitive!(bool);
rust_option_shims_for_primitive!(u8);
rust_option_shims_for_primitive!(u16);
rust_option_shims_for_primitive!(u32);
rust_option_shims_for_primitive!(u64);
rust_option_shims_for_primitive!(usize);
rust_option_shims_for_primitive!(i8);
rust_option_shims_for_primitive!(i16);
rust_option_shims_for_primitive!(i32);
rust_option_shims_for_primitive!(i64);
rust_option_shims_for_primitive!(isize);
rust_option_shims_for_primitive!(f32);
rust_option_shims_for_primitive!(f64);

rust_option_shims!("char", c_char);
rust_option_shims!("string", RustString);
rust_option_shims!("str", &str);
