use crate::syntax::set::UnorderedSet as Set;
use lazy_static::lazy_static;
use std::sync::{Mutex, PoisonError};

#[derive(Copy, Clone, Default)]
pub struct InternedString(&'static str);

impl InternedString {
    pub fn str(self) -> &'static str {
        self.0
    }
}

pub fn intern(s: &str) -> InternedString {
    lazy_static! {
        static ref INTERN: Mutex<Set<&'static str>> = Mutex::new(Set::new());
    }

    let mut set = INTERN.lock().unwrap_or_else(PoisonError::into_inner);
    InternedString(match set.get(s) {
        Some(interned) => *interned,
        None => {
            let interned = Box::leak(Box::from(s));
            set.insert(interned);
            interned
        }
    })
}
