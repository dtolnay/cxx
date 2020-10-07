use std::fmt::{self, Debug};
use std::marker::PhantomData;

pub struct Cfg<'a> {
    pub include_prefix: &'a str,
    marker: PhantomData<*const ()>, // !Send + !Sync
}

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
