//! Less used details of `CxxVector` are exposed in this module. `CxxVector`
//! itself is exposed at the crate root.

pub use crate::cxx_vector::{Iter, IterMut};
#[doc(no_inline)]
pub use crate::CxxVector;
#[doc(inline)]
pub use crate::Vector;
