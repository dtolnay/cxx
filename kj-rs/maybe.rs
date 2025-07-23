use repr::Maybe;
use std::mem::MaybeUninit;

/// # Safety
/// This trait should only be implemented in `workerd-cxx` on types
/// which contain a specialization of `kj::Maybe` that needs to be represented in
/// Rust.
///
/// This trait represents types which have a "niche", a value which represents
/// an invalid instance of the type or can reasonably be interpreted as the absence
/// of that type. This trait is implmented for 2 types, references and `Owns`.
///
/// References have a niche where they are null. It's invalid and ensured by the
/// compiler that this is impossible, so we can optimize an optional type by
/// eliminating a flag that checks whether the item is set or not, and instead
/// checking if it is null.
///
/// `Own`s have a niche where the pointer to the owned data is null. This is
/// a valid instance of `Own`, but was decided by the `kj` authors to represent
/// `kj::none`. In Rust, it is guaranteed that an `Own` is nonnull, requiring
/// `Maybe<Own<T>>` to represent a null `Own`.
///
/// Pointers are not optimized in this way, as `null` is a valid and meaningful
/// instance of a pointer.
///
/// An invalid implementation of this trait for any of the 3 types it is for
/// could result in undefined behavior when passed between languages.
unsafe trait HasNiche: Sized {
    fn is_niche(value: *const Self) -> bool;
}

// In Rust, references are not allowed to be null, so a null `MaybeUninit<&T>` is a niche
unsafe impl<T> HasNiche for &T {
    fn is_niche(value: *const &T) -> bool {
        unsafe {
            // We must cast it as pointing to a pointer, as opposed to a reference,
            // because the rust compiler assumes a reference is never null, and
            // therefore will optimize any null check on that reference.
            (*(value.cast::<*const T>())).is_null()
        }
    }
}

unsafe impl<T> HasNiche for &mut T {
    fn is_niche(value: *const &mut T) -> bool {
        unsafe {
            // We must cast it as pointing to a pointer, as opposed to a reference,
            // because the rust compiler assumes a reference is never null, and
            // therefore will optimize any null check on that reference.
            (*(value.cast::<*mut T>())).is_null()
        }
    }
}

// In `kj`, `kj::Own<T>` are considered `none` in a `Maybe` if the data pointer is null
unsafe impl<T> HasNiche for crate::repr::Own<T> {
    fn is_niche(value: *const crate::repr::Own<T>) -> bool {
        unsafe { (*value).as_ptr().is_null() }
    }
}

/// Trait that is used as the bounds for what can be in a `kj_rs::Maybe`.
///
/// # Safety
/// This trait should only be implemented from macro expansion and should
/// never be manually implemented. An unsound implementation of this trait
/// could result in undefined behavior when passed between languages.
///
/// This trait contains all behavior we need to implement `Maybe<T: MaybeItem>`
/// for every `T` we use, and additionally determines the type layout of
/// the `Maybe<T>`. The only information we can know about `T` comes from
/// this trait, so it must be capable of handling all behavior we want in
/// `kj_rs::Maybe`.
///
/// Every function without a default depends on `MaybeItem::Discriminant`
/// and whether or not `T` implements [`HasNiche`]. Functions with defaults
/// use those functions to implement shared behavior, and simplfy the actual
/// `Maybe<T>` implementation.
pub unsafe trait MaybeItem: Sized {
    type Discriminant: Copy;
    const NONE: Maybe<Self>;
    fn some(value: Self) -> Maybe<Self>;
    fn is_some(value: &Maybe<Self>) -> bool;
    fn is_none(value: &Maybe<Self>) -> bool;
    fn from_option(value: Option<Self>) -> Maybe<Self> {
        match value {
            None => <Self as MaybeItem>::NONE,
            Some(val) => <Self as MaybeItem>::some(val),
        }
    }
    fn drop_in_place(value: &mut Maybe<Self>) {
        if <Self as MaybeItem>::is_some(value) {
            unsafe {
                value.some.assume_init_drop();
            }
        }
    }
}

/// Macro to implement [`MaybeItem`] for `T` which implment [`HasNiche`].
/// Avoids running into generic specialization problems.
macro_rules! impl_maybe_item_for_has_niche {
    ($ty:ty) => {
        unsafe impl<T> MaybeItem for $ty {
            type Discriminant = ();

            fn is_some(value: &Maybe<Self>) -> bool {
                !<$ty as HasNiche>::is_niche(value.some.as_ptr())
            }

            fn is_none(value: &Maybe<Self>) -> bool {
                <$ty as HasNiche>::is_niche(value.some.as_ptr())
            }

            const NONE: Maybe<Self> = {
                Maybe {
                    is_set: (),
                    some: MaybeUninit::zeroed(),
                }
            };

            fn some(value: Self) -> Maybe<Self> {
                Maybe {
                    is_set: (),
                    some: MaybeUninit::new(value)
                }
            }
        }
    };
    ($ty:ty, $($tail:ty),+) => {
        impl_maybe_item_for_has_niche!($ty);
        impl_maybe_item_for_has_niche!($($tail),*);
    };
}

/// Macro to implement [`MaybeItem`] for primitives
/// Avoids running into generic specialization problems.
macro_rules! impl_maybe_item_for_primitive {
    ($ty:ty) => {
        unsafe impl MaybeItem for $ty {
            type Discriminant = bool;

            fn is_some(value: &Maybe<Self>) -> bool {
                value.is_set
            }

            fn is_none(value: &Maybe<Self>) -> bool {
                !value.is_set
            }

            const NONE: Maybe<Self> = {
                Maybe {
                    is_set: false,
                    some: MaybeUninit::uninit(),
                }
            };

            fn some(value: Self) -> Maybe<Self> {
                Maybe {
                    is_set: true,
                    some: MaybeUninit::new(value)
                }
            }
        }
    };
    ($ty:ty, $($tail:ty),+) => {
        impl_maybe_item_for_primitive!($ty);
        impl_maybe_item_for_primitive!($($tail),*);
    };
}

impl_maybe_item_for_has_niche!(crate::Own<T>, &T, &mut T);
impl_maybe_item_for_primitive!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool
);

pub(crate) mod repr {
    use super::MaybeItem;
    use static_assertions::assert_eq_size;
    use std::fmt::Debug;
    use std::mem::MaybeUninit;

    /// A [`Maybe`] represents bindings to the `kj::Maybe` class.
    /// It is an optional type, but represented using a struct, for alignment with kj.
    ///
    /// # Layout
    /// In kj, `Maybe` has 3 specializations, one without niche value optimization, and
    /// two with it. In order to maintain an identical layout in Rust, we include an associated type
    /// in the [`MaybeItem`] trait, which determines the discriminant of the `Maybe<T: MaybeItem>`.
    ///
    /// ## Niche Value Optimization
    /// This discriminant is used in tandem with the [`crate::maybe::HasNiche`] to implement
    /// [`MaybeItem`] properly for values which have a niche, which use a discriminant of [`()`],
    /// the unit type. All other types use [`bool`].
    #[repr(C)]
    pub struct Maybe<T: MaybeItem> {
        pub(super) is_set: T::Discriminant,
        pub(super) some: MaybeUninit<T>,
    }

    assert_eq_size!(Maybe<isize>, [usize; 2]);
    assert_eq_size!(Maybe<&isize>, usize);
    assert_eq_size!(Maybe<crate::Own<isize>>, [usize; 2]);

    impl<T: MaybeItem> Maybe<T> {
        /// # Safety
        /// This function shouldn't be used except by macro generation.
        pub unsafe fn is_set(&self) -> T::Discriminant {
            self.is_set
        }

        /// # Safety
        /// This function shouldn't be used except by macro generation.
        #[inline]
        pub const unsafe fn from_parts_unchecked(
            is_set: T::Discriminant,
            some: MaybeUninit<T>,
        ) -> Maybe<T> {
            Maybe { is_set, some }
        }

        pub fn is_some(&self) -> bool {
            T::is_some(self)
        }

        pub fn is_none(&self) -> bool {
            T::is_none(self)
        }

        // # CONSTRUCTORS
        // These emulate Rust's enum api, which offers constructors for each variant.
        // This mean matching cases, syntax, and behavior.
        // The only place this may be an issue is pattern matching, which will not work,
        // but should produce an error.
        //
        // The following fails to compile:
        // ```{rust,compile_fail}
        // match maybe {
        //     Maybe::Some(_) => ...,
        //     Maybe::None => ...,
        // }
        // ```

        /// The [`Maybe::Some`] function serves the same purpose as an enum constructor.
        ///
        /// Constructing a `Maybe<T>::Some(val)` should only be possible with a valid
        /// instance of `T` from Rust.
        #[allow(non_snake_case)]
        pub fn Some(value: T) -> Maybe<T> {
            T::some(value)
        }

        /// [`Maybe::None`] functions as a constructor for the none variant. It uses
        /// a `const` instead of a function to match syntax with normal Rust enums.
        ///
        /// Constructing a `Maybe<T>::None` variant should always be possible from Rust.
        #[allow(non_upper_case_globals, dead_code)]
        pub const None: Maybe<T> = T::NONE;
    }

    impl<T: MaybeItem> From<Maybe<T>> for Option<T> {
        fn from(value: Maybe<T>) -> Self {
            if value.is_some() {
                // We can't move out of value so we copy it and forget it in
                // order to perform a "manual" move out of value
                let ret = unsafe { Some(value.some.assume_init_read()) };
                std::mem::forget(value);
                ret
            } else {
                None
            }
        }
    }

    impl<T: MaybeItem> From<Option<T>> for Maybe<T> {
        fn from(value: Option<T>) -> Self {
            <T as MaybeItem>::from_option(value)
        }
    }

    impl<T: MaybeItem + Debug> Debug for Maybe<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            if self.is_none() {
                write!(f, "Maybe::None")
            } else {
                write!(f, "Maybe::Some({:?})", unsafe {
                    self.some.assume_init_ref()
                })
            }
        }
    }

    impl<T: MaybeItem> Default for Maybe<T> {
        fn default() -> Self {
            T::NONE
        }
    }

    impl<T: MaybeItem> Drop for Maybe<T> {
        fn drop(&mut self) {
            T::drop_in_place(self);
        }
    }
}
