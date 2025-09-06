#![allow(missing_docs)]

use core::marker::{PhantomData, Unpin};
use core::ops::Deref;

pub unsafe trait RustType {}
pub unsafe trait ImplBox {}
pub unsafe trait ImplVec {}

// Opaque Rust types are required to be Unpin.
pub fn require_unpin<T: ?Sized + Unpin>() {}

pub fn require_box<T: ImplBox>() {}
pub fn require_vec<T: ImplVec>() {}

pub struct With<T: ?Sized>(PhantomData<T>);
pub struct Without<T: ?Sized>(PhantomData<T>);

pub const fn with<T: ?Sized>() -> With<T> {
    With(PhantomData)
}

impl<T: ?Sized + Unpin> With<T> {
    #[allow(clippy::unused_self)]
    pub const fn check_unpin<U>(&self) {}
}

impl<T: ?Sized> Deref for With<T> {
    type Target = Without<T>;
    fn deref(&self) -> &Self::Target {
        &Without(PhantomData)
    }
}

impl<T: ?Sized> Without<T> {
    #[allow(clippy::unused_self)]
    pub const fn check_unpin<U: ReferenceToUnpin>(&self) {}
}

#[diagnostic::on_unimplemented(
    message = "mutable reference to C++ type requires a pin -- use Pin<{Self}>",
    label = "use Pin<{Self}>"
)]
pub trait ReferenceToUnpin {}
