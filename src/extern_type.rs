use crate::kind::{Kind, Opaque};
use crate::CxxString;
use crate::ExternType;

#[doc(hidden)]
pub fn verify_extern_type<T: ExternType<Id = Id>, Id>() {}

#[doc(hidden)]
pub fn verify_extern_kind<T: ExternType<Kind = Kind>, Kind: self::Kind>() {}

macro_rules! impl_extern_type {
    ($([$kind:ident] $($ty:path = $cxxpath:literal)*)*) => {
        $($(
            unsafe impl ExternType for $ty {
                #[doc(hidden)]
                type Id = crate::type_id!($cxxpath);
                type Kind = $kind;
            }
        )*)*
    };
}

impl_extern_type! {
    [Opaque]
    CxxString = "std::string"
}
