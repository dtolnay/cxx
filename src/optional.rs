//! Less used details of `CxxOptional`.
//!
//! `CxxOptional` itself is exposed at the crate root.

pub use crate::cxx_optional::{Iter, IterMut, OptionalElement};
#[doc(inline)]
pub use crate::Optional;
#[doc(no_inline)]
pub use cxx::CxxOptional;
