use crate::c_char::c_char;
use crate::rust_option::RustOption;
use core::mem;
use core::ptr;

macro_rules! rust_option_shims {
    ($segment:expr, $ty:ty) => {
        const_assert_eq!(
            mem::size_of::<Option<&$ty>>(),
            mem::size_of::<RustOption<&$ty>>()
        );
        const_assert_eq!(mem::size_of::<Option<&$ty>>(), mem::size_of::<usize>());

        const _: () = {
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$new")]
            unsafe extern "C" fn __const_new(this: *mut RustOption<&$ty>) {
                unsafe { ptr::write(this, RustOption::new()) };
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$drop")]
            unsafe extern "C" fn __const_drop(this: *mut RustOption<&$ty>) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$has_value")]
            unsafe extern "C" fn __const_has_value(this: *mut RustOption<&$ty>) -> bool {
                unsafe { this.as_ref().unwrap().value().is_some() }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$value_ptr")]
            unsafe extern "C" fn __const_value_ptr(this: *mut RustOption<&$ty>) -> *mut &$ty {
                unsafe { this.as_mut().unwrap().as_ref_mut_inner_unchecked() as _ }
            }
            #[export_name = concat!("cxxbridge1$rust_option$const$", $segment, "$set")]
            unsafe extern "C" fn __const_set<'__cxx>(
                this: *mut RustOption<&'__cxx $ty>,
                value: *mut &'__cxx $ty,
            ) {
                unsafe { this.as_mut().unwrap().set(*value) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$new")]
            unsafe extern "C" fn __new(this: *mut RustOption<&mut $ty>) {
                unsafe { ptr::write(this, RustOption::new()) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$drop")]
            unsafe extern "C" fn __drop(this: *mut RustOption<&mut $ty>) {
                unsafe { ptr::drop_in_place(this) }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$has_value")]
            unsafe extern "C" fn __has_value(this: *mut RustOption<&mut $ty>) -> bool {
                unsafe { this.as_ref().unwrap().value().is_some() }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$value_ptr")]
            unsafe extern "C" fn __value_ptr(this: *mut RustOption<&mut $ty>) -> *mut &mut $ty {
                unsafe { this.as_mut().unwrap().as_ref_mut_inner_unchecked() as _ }
            }
            #[export_name = concat!("cxxbridge1$rust_option$", $segment, "$set")]
            unsafe extern "C" fn __set<'__cxx>(
                this: *mut RustOption<&'__cxx mut $ty>,
                value: *mut &'__cxx mut $ty,
            ) {
                unsafe { this.as_mut().unwrap().set(*value) }
            }
        };
    };
}

macro_rules! rust_option_shims_for_primitive {
    ($ty:ident) => {
        rust_option_shims!(stringify!($ty), $ty);
    };
}

rust_option_shims_for_primitive!(u8);
rust_option_shims_for_primitive!(bool);
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
