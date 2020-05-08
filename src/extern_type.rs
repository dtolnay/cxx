pub unsafe trait ExternType {
    type Id;
}

#[doc(hidden)]
pub fn verify_extern_type<T: ExternType<Id = Id>, Id>() {}
