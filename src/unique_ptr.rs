use crate::cxx_vector::{CxxVector, VectorElement};
use crate::fmt::display;
use crate::kind::Trivial;
use crate::string::CxxString;
use crate::ExternType;
use core::ffi::c_void;
use core::fmt::{self, Debug, Display};
use core::marker::PhantomData;
use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::pin::Pin;
use cxx::type_id;

/// Rust representation of the default deleter of std::unique_ptr, `std::default_delete<T>`
#[derive(Default, Copy, Clone, Debug)]
pub struct DefaultDeleter([u8; 0]);

unsafe impl ExternType for DefaultDeleter {
    type Id = type_id!("std::default_delete");
    type Kind = Trivial;
}

/// Binding to C++ `std::unique_ptr<T, D>`.
///
/// This representation assumes that the deleter is
/// 1. the first member of the smart pointer.
/// 2. optimized away by some form of empty base class optimization in case the deleter is stateless
///    (which is the case for the default deleter). For this, a zero-sized type is used on the Rust
///    side.
/// 3. trivial to copy and move. This is trivially true for all stateless deleters as well as
///    deleters that just contain references/pointers (e.g. when using an std::pmr::memory_resource)
#[repr(C)]
pub struct UniquePtr<T, D = DefaultDeleter>
where
    T: UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    deleter: D,
    repr: MaybeUninit<*mut c_void>,
    ty: PhantomData<T>,
}

impl<T, D> UniquePtr<T, D>
where
    T: UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    /// Makes a new UniquePtr wrapping a null pointer.
    ///
    /// Matches the behavior of default-constructing a std::unique\_ptr.
    pub fn null() -> Self
    where
        D: Default,
    {
        UniquePtr {
            deleter: D::default(),
            repr: T::__null(),
            ty: PhantomData,
        }
    }

    /// Allocates memory on the heap and makes a UniquePtr pointing to it.
    ///
    /// The deleter will be default-constructed. Please note that the memory is allocated using
    /// operator ::new. The deleter needs to be able to handle such a pointer on deletion.
    pub fn new(value: T) -> Self
    where
        T: ExternType<Kind = Trivial>,
        D: Default,
    {
        UniquePtr {
            deleter: D::default(),
            repr: T::__new(value),
            ty: PhantomData,
        }
    }

    /// Allocates memory on the heap and makes a UniquePtr pointing to it.
    ///
    /// Please note that the memory is allocated using operator ::new. The deleter needs to be able
    /// to handle such a pointer on deletion.
    pub fn with_deleter(value: T, deleter: D) -> Self
    where
        T: ExternType<Kind = Trivial>,
    {
        UniquePtr {
            deleter,
            repr: T::__new(value),
            ty: PhantomData,
        }
    }

    /// Checks whether the UniquePtr does not own an object.
    ///
    /// This is the opposite of [std::unique_ptr\<T\>::operator bool](https://en.cppreference.com/w/cpp/memory/unique_ptr/operator_bool).
    pub fn is_null(&self) -> bool {
        let ptr = unsafe { T::__get(self.repr) };
        ptr.is_null()
    }

    /// Returns a reference to the object owned by this UniquePtr if any,
    /// otherwise None.
    pub fn as_ref(&self) -> Option<&T> {
        unsafe { T::__get(self.repr).as_ref() }
    }

    /// Returns a mutable pinned reference to the object owned by this UniquePtr
    /// if any, otherwise None.
    pub fn as_mut(&mut self) -> Option<Pin<&mut T>> {
        unsafe {
            let mut_reference = (T::__get(self.repr) as *mut T).as_mut()?;
            Some(Pin::new_unchecked(mut_reference))
        }
    }

    /// Returns a mutable pinned reference to the object owned by this
    /// UniquePtr.
    ///
    /// # Panics
    ///
    /// Panics if the UniquePtr holds a null pointer.
    pub fn pin_mut(&mut self) -> Pin<&mut T> {
        match self.as_mut() {
            Some(target) => target,
            None => panic!(
                "called pin_mut on a null UniquePtr<{}>",
                display(T::__typename),
            ),
        }
    }

    /// Consumes the UniquePtr, releasing its ownership of the heap-allocated T.
    ///
    /// Matches the behavior of [std::unique_ptr\<T\>::release](https://en.cppreference.com/w/cpp/memory/unique_ptr/release).
    pub fn into_raw(self) -> *mut T {
        let ptr = unsafe { T::__release(self.repr) };
        mem::forget(self);
        ptr
    }

    /// Constructs a UniquePtr retaking ownership of a pointer previously
    /// obtained from `into_raw`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory
    /// problems. For example a double-free may occur if the function is called
    /// twice on the same raw pointer. In addition, the caller must make sure
    /// that the used deleter is able to handle the given pointer when default
    /// constructed.
    pub unsafe fn from_raw(raw: *mut T) -> Self
    where
        D: Default,
    {
        UniquePtr {
            deleter: D::default(),
            repr: unsafe { T::__raw(raw) },
            ty: PhantomData,
        }
    }

    /// Constructs a UniquePtr retaking ownership of a pointer previously
    /// obtained from `into_raw`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because improper use may lead to memory
    /// problems. For example a double-free may occur if the function is called
    /// twice on the same raw pointer. In addition, the given deleter needs to
    /// be able to handle the pointer.
    pub unsafe fn from_raw_with_deleter(raw: *mut T, deleter: D) -> Self {
        UniquePtr {
            deleter,
            repr: unsafe { T::__raw(raw) },
            ty: PhantomData,
        }
    }

    /// Returns a reference to the deleter used during destruction or releasing of the pointee.
    pub fn get_deleter(&self) -> &D {
        &self.deleter
    }
}

unsafe impl<T, D> Send for UniquePtr<T, D>
where
    T: Send + UniquePtrTarget<D>,
    D: Send + ExternType<Kind = Trivial>,
{
}

unsafe impl<T, D> Sync for UniquePtr<T, D>
where
    T: Sync + UniquePtrTarget<D>,
    D: Sync + ExternType<Kind = Trivial>,
{
}

// UniquePtr is not a self-referential type and is safe to move out of a Pin,
// regardless whether the pointer's target is Unpin.
impl<T, D> Unpin for UniquePtr<T, D>
where
    T: UniquePtrTarget<D>,
    D: Unpin + ExternType<Kind = Trivial>,
{
}

impl<T, D> Drop for UniquePtr<T, D>
where
    T: UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    fn drop(&mut self) {
        unsafe { T::__drop(self) }
    }
}

impl<T, D> Deref for UniquePtr<T, D>
where
    T: UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.as_ref() {
            Some(target) => target,
            None => panic!(
                "called deref on a null UniquePtr<{}>",
                display(T::__typename),
            ),
        }
    }
}

impl<T, D> DerefMut for UniquePtr<T, D>
where
    T: UniquePtrTarget<D> + Unpin,
    D: ExternType<Kind = Trivial>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.as_mut() {
            Some(target) => Pin::into_inner(target),
            None => panic!(
                "called deref_mut on a null UniquePtr<{}>",
                display(T::__typename),
            ),
        }
    }
}

impl<T, D> Debug for UniquePtr<T, D>
where
    T: Debug + UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Debug::fmt(value, formatter),
        }
    }
}

impl<T, D> Display for UniquePtr<T, D>
where
    T: Display + UniquePtrTarget<D>,
    D: ExternType<Kind = Trivial>,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self.as_ref() {
            None => formatter.write_str("nullptr"),
            Some(value) => Display::fmt(value, formatter),
        }
    }
}

/// Trait bound for types which may be used as the `T` inside of a
/// `UniquePtr<T>` in generic code.
///
/// This trait has no publicly callable or implementable methods. Implementing
/// it outside of the CXX codebase is not supported.
///
/// # Example
///
/// A bound `T: UniquePtrTarget` may be necessary when manipulating
/// [`UniquePtr`] in generic code.
///
/// ```
/// use cxx::memory::{UniquePtr, UniquePtrTarget};
/// use std::fmt::Display;
///
/// pub fn take_generic_ptr<T>(ptr: UniquePtr<T>)
/// where
///     T: UniquePtrTarget + Display,
/// {
///     println!("the unique_ptr points to: {}", *ptr);
/// }
/// ```
///
/// Writing the same generic function without a `UniquePtrTarget` trait bound
/// would not compile.
pub unsafe trait UniquePtrTarget<D = DefaultDeleter> {
    #[doc(hidden)]
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result;
    #[doc(hidden)]
    fn __null() -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    fn __new(value: Self) -> MaybeUninit<*mut c_void>
    where
        Self: Sized,
    {
        // Opaque C types do not get this method because they can never exist by
        // value on the Rust side of the bridge.
        let _ = value;
        unreachable!()
    }
    #[doc(hidden)]
    unsafe fn __raw(raw: *mut Self) -> MaybeUninit<*mut c_void>;
    #[doc(hidden)]
    unsafe fn __get(repr: MaybeUninit<*mut c_void>) -> *const Self;
    #[doc(hidden)]
    unsafe fn __release(repr: MaybeUninit<*mut c_void>) -> *mut Self;
    #[doc(hidden)]
    unsafe fn __drop(ptr: &mut UniquePtr<Self, D>)
    where
        Self: Sized,
        D: ExternType<Kind = Trivial>;
}

extern "C" {
    #[link_name = "cxxbridge1$unique_ptr$std$string$null"]
    fn unique_ptr_std_string_null(this: *mut MaybeUninit<*mut c_void>);
    #[link_name = "cxxbridge1$unique_ptr$std$string$raw"]
    fn unique_ptr_std_string_raw(this: *mut MaybeUninit<*mut c_void>, raw: *mut CxxString);
    #[link_name = "cxxbridge1$unique_ptr$std$string$get"]
    fn unique_ptr_std_string_get(this: *const MaybeUninit<*mut c_void>) -> *const CxxString;
    #[link_name = "cxxbridge1$unique_ptr$std$string$release"]
    fn unique_ptr_std_string_release(this: *mut MaybeUninit<*mut c_void>) -> *mut CxxString;
    #[link_name = "cxxbridge1$unique_ptr$std$string$drop"]
    fn unique_ptr_std_string_drop(this: *mut MaybeUninit<*mut c_void>);
}

unsafe impl UniquePtrTarget for CxxString {
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("CxxString")
    }
    fn __null() -> MaybeUninit<*mut c_void> {
        let mut repr = MaybeUninit::uninit();
        unsafe {
            unique_ptr_std_string_null(&mut repr);
        }
        repr
    }
    unsafe fn __raw(raw: *mut Self) -> MaybeUninit<*mut c_void> {
        let mut repr = MaybeUninit::uninit();
        unsafe { unique_ptr_std_string_raw(&mut repr, raw) }
        repr
    }
    unsafe fn __get(repr: MaybeUninit<*mut c_void>) -> *const Self {
        unsafe { unique_ptr_std_string_get(&repr) }
    }
    unsafe fn __release(mut repr: MaybeUninit<*mut c_void>) -> *mut Self {
        unsafe { unique_ptr_std_string_release(&mut repr) }
    }
    unsafe fn __drop(ptr: &mut UniquePtr<Self>)
    where
        Self: Sized,
    {
        unsafe { unique_ptr_std_string_drop(&mut ptr.repr) }
    }
}

unsafe impl<T> UniquePtrTarget for CxxVector<T>
where
    T: VectorElement,
{
    fn __typename(f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CxxVector<{}>", display(T::__typename))
    }
    fn __null() -> MaybeUninit<*mut c_void> {
        T::__unique_ptr_null()
    }
    unsafe fn __raw(raw: *mut Self) -> MaybeUninit<*mut c_void> {
        unsafe { T::__unique_ptr_raw(raw) }
    }
    unsafe fn __get(repr: MaybeUninit<*mut c_void>) -> *const Self {
        unsafe { T::__unique_ptr_get(repr) }
    }
    unsafe fn __release(repr: MaybeUninit<*mut c_void>) -> *mut Self {
        unsafe { T::__unique_ptr_release(repr) }
    }
    unsafe fn __drop(ptr: &mut UniquePtr<Self>)
    where
        Self: Sized,
    {
        unsafe { T::__unique_ptr_drop(ptr.repr) }
    }
}
