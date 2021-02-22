/// For use in impls of the `ExternType` trait. See [`ExternType`].
///
/// [`ExternType`]: trait.ExternType.html
#[macro_export]
macro_rules! type_id {
    ($($path:tt)*) => {
        $crate::private::type_id! { $crate $($path)* }
    };
}
