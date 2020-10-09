use std::fmt::{self, Debug};
use std::marker::PhantomData;

/// Build configuration. See [CFG].
pub struct Cfg<'a> {
    pub include_prefix: &'a str,
    marker: PhantomData<*const ()>, // !Send + !Sync
}

/// Global configuration of the current build.
///
/// <br>
///
/// ## **`CFG.include_prefix`**
///
/// Presently the only exposed configuration is the `include_prefix`, the prefix
/// at which C++ code from your crate as well as directly dependent crates can
/// access the code generated during this build.
///
/// By default, the `include_prefix` is equal to the name of the current crate.
/// That means if our crate is called `demo` and has Rust source files in a
/// *src/* directory and maybe some handwritten C++ header files in an
/// *include/* directory, then the current crate as well as downstream crates
/// might include them as follows:
///
/// ```
/// # const _: &str = stringify! {
///   // include one of the handwritten headers:
/// #include "demo/include/wow.h"
///
///   // include a header generated from Rust cxx::bridge:
/// #include "demo/src/lib.rs.h"
/// # };
/// ```
///
/// By modifying `CFG.include_prefix` we can substitute a prefix that is
/// different from the crate name if desired. Here we'll change it to
/// `"path/to"` which will make import paths take the form
/// `"path/to/include/wow.h"` and `"path/to/src/lib.rs.h"`.
///
/// ```no_run
/// // build.rs
///
/// use cxx_build::CFG;
///
/// fn main() {
///     CFG.include_prefix = "path/to";
///
///     cxx_build::bridge("src/lib.rs")
///         .file("src/demo.cc") // probably contains `#include "path/to/src/lib.rs.h"`
///         /* ... */
///         .compile("demo");
/// }
/// ```
///
/// Note that cross-crate imports are only made available between **direct
/// dependencies**. Another crate must directly depend on your crate in order to
/// #include its headers; a transitive dependency is not sufficient.
/// Additionally, headers from a direct dependency are only importable if the
/// dependency's Cargo.toml manifest contains a `links` key. If not, its headers
/// will not be importable from outside of the same crate.
#[cfg(doc)]
pub static mut CFG: Cfg = Cfg {
    include_prefix: "",
    marker: PhantomData,
};

impl<'a> Debug for Cfg<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter
            .debug_struct("Cfg")
            .field("include_prefix", &self.include_prefix)
            .finish()
    }
}

#[cfg(not(doc))]
pub use self::r#impl::Cfg::CFG;

#[cfg(not(doc))]
mod r#impl {
    use lazy_static::lazy_static;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::fmt::{self, Debug};
    use std::marker::PhantomData;
    use std::ops::{Deref, DerefMut};
    use std::sync::{PoisonError, RwLock};

    lazy_static! {
        static ref PACKAGE_NAME: Box<str> = {
            crate::env_os("CARGO_PKG_NAME")
                .map(|pkg| pkg.to_string_lossy().into_owned().into_boxed_str())
                .unwrap_or_default()
        };
        static ref INCLUDE_PREFIX: RwLock<Vec<&'static str>> = RwLock::new(vec![&PACKAGE_NAME]);
    }

    thread_local! {
        // FIXME: If https://github.com/rust-lang/rust/issues/77425 is resolved,
        // we can delete this thread local side table and instead make each CFG
        // instance directly own the associated super::Cfg.
        //
        //     #[allow(const_item_mutation)]
        //     pub const CFG: Cfg = Cfg {
        //         cfg: AtomicPtr::new(ptr::null_mut()),
        //     };
        //     pub struct Cfg {
        //         cfg: AtomicPtr<super::Cfg>,
        //     }
        //
        static CONST_DEREFS: RefCell<HashMap<Handle, Box<super::Cfg<'static>>>> = RefCell::default();
    }

    #[derive(Eq, PartialEq, Hash)]
    struct Handle(*const Cfg<'static>);

    impl<'a> Cfg<'a> {
        fn current() -> super::Cfg<'a> {
            let include_prefix = *INCLUDE_PREFIX
                .read()
                .unwrap_or_else(PoisonError::into_inner)
                .last()
                .unwrap();
            super::Cfg {
                include_prefix,
                marker: PhantomData,
            }
        }

        const fn handle(self: &Cfg<'a>) -> Handle {
            Handle(<*const Cfg>::cast(self))
        }
    }

    // Since super::Cfg is !Send and !Sync, all Cfg are thread local and will
    // drop on the same thread where they were created.
    pub enum Cfg<'a> {
        Mut(super::Cfg<'a>),
        CFG,
    }

    impl<'a> Debug for Cfg<'a> {
        fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            if let Cfg::Mut(cfg) = self {
                Debug::fmt(cfg, formatter)
            } else {
                Debug::fmt(&Cfg::current(), formatter)
            }
        }
    }

    impl<'a> Deref for Cfg<'a> {
        type Target = super::Cfg<'a>;

        fn deref(&self) -> &Self::Target {
            if let Cfg::Mut(cfg) = self {
                cfg
            } else {
                let cfg = CONST_DEREFS.with(|derefs| -> *mut super::Cfg {
                    &mut **derefs
                        .borrow_mut()
                        .entry(self.handle())
                        .or_insert_with(|| Box::new(Cfg::current()))
                });
                unsafe { &mut *cfg }
            }
        }
    }

    impl<'a> DerefMut for Cfg<'a> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            if let Cfg::CFG = self {
                CONST_DEREFS.with(|derefs| derefs.borrow_mut().remove(&self.handle()));
                *self = Cfg::Mut(Cfg::current());
            }
            match self {
                Cfg::Mut(cfg) => cfg,
                Cfg::CFG => unreachable!(),
            }
        }
    }

    impl<'a> Drop for Cfg<'a> {
        fn drop(&mut self) {
            if let Cfg::Mut(cfg) = self {
                INCLUDE_PREFIX
                    .write()
                    .unwrap_or_else(PoisonError::into_inner)
                    .push(Box::leak(Box::from(cfg.include_prefix)));
            } else {
                CONST_DEREFS.with(|derefs| derefs.borrow_mut().remove(&self.handle()));
            }
        }
    }
}
