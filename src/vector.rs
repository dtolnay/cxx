//! Less used details of `CxxVector` are exposed in this module. `CxxVector`
//! itself is exposed at the crate root.

pub use crate::cxx_vector::{Iter, IterMut};
#[doc(inline)]
pub use crate::Vector;
#[doc(no_inline)]
pub use cxx::CxxVector;
