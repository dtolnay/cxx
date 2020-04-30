use std::mem;
use std::ptr;

#[repr(C)]
pub struct RustVec<T> {
    repr: Vec<T>,
}

macro_rules! attr {
    (#[$name:ident = $value:expr] $($rest:tt)*) => {
        #[$name = $value]
        $($rest)*
    };
}

macro_rules! rust_vec_shims_for_primitive {
    ($ty:ident) => {
        const_assert_eq!(mem::size_of::<[usize; 3]>(), mem::size_of::<Vec<$ty>>());
        const_assert_eq!(mem::align_of::<usize>(), mem::align_of::<Vec<$ty>>());

        const _: () = {
            attr! {
                #[export_name = concat!("cxxbridge03$rust_vec$", stringify!($ty), "$new")]
                unsafe extern "C" fn __new(this: *mut RustVec<$ty>) {
                    ptr::write(this, RustVec { repr: Vec::new() });
                }
            }
            attr! {
                #[export_name = concat!("cxxbridge03$rust_vec$", stringify!($ty), "$drop")]
                unsafe extern "C" fn __drop(this: *mut RustVec<$ty>) {
                    ptr::drop_in_place(this);
                }
            }
            attr! {
                #[export_name = concat!("cxxbridge03$rust_vec$", stringify!($ty), "$len")]
                unsafe extern "C" fn __len(this: *const RustVec<$ty>) -> usize {
                    (*this).repr.len()
                }
            }
            attr! {
                #[export_name = concat!("cxxbridge03$rust_vec$", stringify!($ty), "$data")]
                unsafe extern "C" fn __data(this: *const RustVec<$ty>) -> *const $ty {
                    (*this).repr.as_ptr()
                }
            }
            attr! {
                #[export_name = concat!("cxxbridge03$rust_vec$", stringify!($ty), "$stride")]
                unsafe extern "C" fn __stride() -> usize {
                    mem::size_of::<$ty>()
                }
            }
        };
    };
}

rust_vec_shims_for_primitive!(bool);
rust_vec_shims_for_primitive!(u8);
rust_vec_shims_for_primitive!(u16);
rust_vec_shims_for_primitive!(u32);
rust_vec_shims_for_primitive!(u64);
rust_vec_shims_for_primitive!(i8);
rust_vec_shims_for_primitive!(i16);
rust_vec_shims_for_primitive!(i32);
rust_vec_shims_for_primitive!(i64);
rust_vec_shims_for_primitive!(f32);
rust_vec_shims_for_primitive!(f64);
