use core::marker::{PhantomData};
use std::mem::MaybeUninit;
use core::ffi::c_void;
use crate::std::fmt;

/// Binding to C++ `std::function<R(A...)`.
///
/// # Invariants
///
/// As an invariant of this API and the static analysis of the cxx::bridge
/// macro, in Rust code we can never obtain a `CxxFunction` by value. Instead in
/// Rust code we will only ever look at a callback behind a reference or smart
/// pointer, as in `&CxxFunction<A, R>` or `UniquePtr<CxxFunction<A, R>>`.
#[repr(C, packed)]
pub struct CxxFunction<A, R> {
    // A thing, because repr(C) structs are not allowed to consist exclusively
    // of PhantomData fields.
    _void: [c_void; 0],
    _func: PhantomData<fn(R) -> A>,
}

impl<A: CxxFunctionArguments<R>, R> CxxFunction<A, R> {

    /// Sends the callback and the arguments to C++ land, and calls it there
    pub fn call(&self, arguments: A) -> R {
        unsafe { A::__call(self, arguments) }
    }
}

/// Trait bound for types which may be used as the `A` inside of a
/// `CxxFunction<A, R>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: CxxFunctionArguments` may be necessary when manipulating
/// [`CxxFunction`] in generic code.
///
/// ```
/// use cxx::CxxFunction;
/// use cxx::private::CxxFunctionArguments;
/// use std::fmt::Display;
///
/// pub fn take_generic_function<A, R>(ptr: &CxxFunction<A, R>, arguments: A)
/// where
///     A: CxxFunctionArguments<R>,
///     R: Display,
/// {
///     let result = ptr.call(arguments);
///     println!("the callback returned: {}", result);
/// }
/// ```
///
/// Writing the same generic function without a `CxxFunctionArguments` trait bound
/// would not compile.
pub unsafe trait CxxFunctionArguments<R>: Sized {
    #[doc(hidden)]
    unsafe fn __call(f: &CxxFunction<Self, R>, arg: Self) -> R;
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    fn __unique_ptr_null() -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_raw(raw: *mut CxxFunction<Self, R>) -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_get(repr: MaybeUninit<*mut c_void>) -> *const CxxFunction<Self, R>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_release(repr: MaybeUninit<*mut c_void>) -> *mut CxxFunction<Self, R>;
    #[doc(hidden)]
    unsafe fn __unique_ptr_drop(repr: MaybeUninit<*mut c_void>);
}
