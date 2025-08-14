use crate::ffi::{OpaqueCxxClass, Shared};
use kj_rs::{
    maybe::MaybeItem,
    repr::{KjMaybe, KjOwn},
};
use std::{cmp::PartialEq, fmt::Debug};

pub fn take_maybe_own_ret(val: KjMaybe<KjOwn<OpaqueCxxClass>>) -> KjMaybe<KjOwn<OpaqueCxxClass>> {
    let mut option: Option<KjOwn<OpaqueCxxClass>> = val.into();
    if let Some(val) = &mut option {
        val.as_mut().set_data(42);
    }

    option.into()
}

pub fn take_maybe_own(val: KjMaybe<KjOwn<OpaqueCxxClass>>) {
    let option: Option<KjOwn<OpaqueCxxClass>> = val.into();
    // Own gets destoyed at end of `if let` block, because it takes ownership of `option`
    if let Some(own) = option {
        assert_eq!(own.get_data(), 42);
    }
}

/// # Safety: Uses a reference in a function that can be called from C++, which is opaque
/// to the Rust compiler, so it cannot verify lifetime requirements
pub unsafe fn take_maybe_ref_ret<'a>(val: KjMaybe<&'a u64>) -> KjMaybe<&'a u64> {
    let option: Option<&u64> = val.into();
    if let Some(num) = &option {
        assert_eq!(**num, 15);
    }
    option.into()
}

pub fn take_maybe_ref(val: KjMaybe<&u64>) {
    let mut option: Option<&u64> = val.into();
    // Pure Rust at this point, but just in case
    if let Some(val) = option.take() {
        assert_eq!(*val, 15);
    }
}

pub fn take_maybe_shared_ret(val: KjMaybe<Shared>) -> KjMaybe<Shared> {
    let mut option: Option<Shared> = val.into();
    if let Some(mut shared) = option.take() {
        shared.i = 18;
    }
    option.into()
}

pub fn take_maybe_shared(val: KjMaybe<Shared>) {
    let _: Option<Shared> = val.into();
}

#[allow(clippy::needless_pass_by_value, dead_code)]
fn test_maybe_some<T: MaybeItem + PartialEq + Debug>(val: KjMaybe<T>, num: T) {
    assert!(val.is_some());
    let opt: Option<T> = val.into();
    assert_eq!(opt.unwrap(), num);
}

#[allow(clippy::needless_pass_by_value, dead_code)]
fn test_maybe_none<T: MaybeItem + PartialEq + Debug>(val: KjMaybe<T>) {
    assert!(val.is_none());
}

#[cfg(test)]
pub mod tests {
    use super::{test_maybe_none, test_maybe_some};
    use crate::ffi::{self, OpaqueCxxClass, Shared};
    use kj_rs::repr::{KjMaybe, KjOwn};

    #[test]
    fn test_some() {
        let maybe: KjMaybe<i64> = ffi::return_maybe();
        assert!(!maybe.is_none());
    }

    #[test]
    fn test_none() {
        let maybe: KjMaybe<i64> = ffi::return_maybe_none();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_none_ref() {
        let maybe = ffi::return_maybe_ref_none();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_none_ref_opt() {
        let maybe = ffi::return_maybe_ref_none();
        let maybe: Option<&i64> = maybe.into();
        assert!(maybe.is_none());
    }

    #[test]
    fn test_some_ref() {
        let maybe = ffi::return_maybe_ref_some();
        assert!(maybe.is_some());
    }

    #[test]
    fn test_some_ref_opt() {
        let maybe = ffi::return_maybe_ref_some();
        let maybe: Option<&i64> = maybe.into();
        assert!(maybe.is_some());
    }

    #[test]
    fn test_some_shared() {
        let maybe: KjMaybe<Shared> = ffi::return_maybe_shared_some();
        assert!(!maybe.is_none());
        let opt: Option<Shared> = maybe.into();
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().i, 14);
    }

    #[test]
    fn test_none_shared() {
        let maybe: KjMaybe<Shared> = ffi::return_maybe_shared_none();
        assert!(maybe.is_none());
        let opt: Option<Shared> = maybe.into();
        assert!(opt.is_none());
    }

    #[test]
    fn test_some_own() {
        let maybe = ffi::return_maybe_own_some();
        assert!(!maybe.is_none());
        let opt: Option<KjOwn<OpaqueCxxClass>> = maybe.into();
        assert!(opt.is_some());
        assert_eq!(opt.unwrap().get_data(), 14);
    }

    #[test]
    fn test_none_own() {
        let maybe = ffi::return_maybe_own_none();
        assert!(maybe.is_none());
        let opt: Option<KjOwn<OpaqueCxxClass>> = maybe.into();
        assert!(opt.is_none());
    }

    #[test]
    fn test_some_own_maybe() {
        let maybe = ffi::return_maybe_own_some();
        assert!(!maybe.is_none());
        assert!(maybe.is_some());
    }

    #[test]
    fn test_none_own_maybe() {
        let maybe = ffi::return_maybe_own_none();
        assert!(maybe.is_none());
        assert!(!maybe.is_some());
    }

    #[test]
    fn test_maybe_cxx_shared() {
        let shared = ffi::Shared { i: -37 };
        let some = KjMaybe::Some(shared);
        ffi::cxx_take_maybe_shared_some(some);
        let none: KjMaybe<Shared> = KjMaybe::None;
        ffi::cxx_take_maybe_shared_none(none);
    }

    #[test]
    fn test_maybe_cxx_ref_shared() {
        let shared = ffi::Shared { i: -38 };
        let some = KjMaybe::Some(&shared);
        ffi::cxx_take_maybe_ref_shared_some(some);
        let none: KjMaybe<&Shared> = KjMaybe::None;
        ffi::cxx_take_maybe_ref_shared_none(none);
    }

    #[test]
    fn test_primitive_types() {
        macro_rules! Maybe {
            ($ty:ty) => {
                let (some, none): (KjMaybe<$ty>, KjMaybe<$ty>) = unsafe {(
                    KjMaybe::from_parts_unchecked(true, std::mem::MaybeUninit::new(<$ty>::default())),
                    KjMaybe::from_parts_unchecked(false, std::mem::MaybeUninit::uninit()),
                )};

                assert!(some.is_some());
                assert!(!some.is_none());
                assert!(!none.is_some());
                assert!(none.is_none());

                let opt: Option<$ty> = some.into();
                assert_eq!(opt.unwrap(), <$ty>::default());
            };
            ($ty:ty, $($tail:ty),+) => {
                Maybe!($ty);
                Maybe!($($tail),*);
            }
        }
        Maybe!(
            u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool
        );

        test_maybe_some(ffi::test_maybe_u8_some(), 234);
        test_maybe_some(ffi::test_maybe_u16_some(), 235);
        test_maybe_some(ffi::test_maybe_u32_some(), 236);
        test_maybe_some(ffi::test_maybe_u64_some(), 237);
        test_maybe_some(ffi::test_maybe_usize_some(), 238);
        test_maybe_some(ffi::test_maybe_i8_some(), 97);
        test_maybe_some(ffi::test_maybe_i16_some(), 240);
        test_maybe_some(ffi::test_maybe_i32_some(), 241);
        test_maybe_some(ffi::test_maybe_i64_some(), 242);
        test_maybe_some(ffi::test_maybe_isize_some(), 243);
        test_maybe_some(ffi::test_maybe_f32_some(), 244.678);
        test_maybe_some(ffi::test_maybe_f64_some(), 245.678);
        test_maybe_some(ffi::test_maybe_bool_some(), false);

        test_maybe_none(ffi::test_maybe_u8_none());
        test_maybe_none(ffi::test_maybe_u16_none());
        test_maybe_none(ffi::test_maybe_u32_none());
        test_maybe_none(ffi::test_maybe_u64_none());
        test_maybe_none(ffi::test_maybe_usize_none());
        test_maybe_none(ffi::test_maybe_i8_none());
        test_maybe_none(ffi::test_maybe_i16_none());
        test_maybe_none(ffi::test_maybe_i32_none());
        test_maybe_none(ffi::test_maybe_i64_none());
        test_maybe_none(ffi::test_maybe_isize_none());
        test_maybe_none(ffi::test_maybe_f32_none());
        test_maybe_none(ffi::test_maybe_f64_none());
        test_maybe_none(ffi::test_maybe_bool_none());
    }

    #[test]
    fn test_pass_cxx() {
        let maybe = ffi::return_maybe_own_some();
        ffi::take_maybe_own_cxx(maybe);
    }

    #[test]
    fn test_pass_rust() {
        let mut own = ffi::cxx_kj_own();
        own.pin_mut().set_data(14);
        let maybe_some: KjMaybe<KjOwn<ffi::OpaqueCxxClass>> = KjMaybe::Some(own);
        let maybe_none: KjMaybe<KjOwn<ffi::OpaqueCxxClass>> = KjMaybe::None;

        assert!(maybe_some.is_some());
        assert!(maybe_none.is_none());

        ffi::take_maybe_own_cxx(maybe_some);
        ffi::take_maybe_own_cxx(maybe_none);
    }

    #[test]
    fn test_maybe_driver() {
        ffi::test_maybe_reference_shared_own_driver();
    }
}
