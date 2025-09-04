#![allow(missing_docs)]

use core::marker::Unpin;

pub unsafe trait RustType {}
pub unsafe trait ImplBox {}
pub unsafe trait ImplVec {}

// Opaque Rust types are required to be Unpin.
pub fn require_unpin<T: ?Sized + Unpin>() {}

pub fn require_box<T: ImplBox>() {}
pub fn require_vec<T: ImplVec>() {}
