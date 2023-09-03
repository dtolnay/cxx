use crate::syntax::set::UnorderedSet as Set;
use once_cell::sync::OnceCell;
use std::sync::{Mutex, PoisonError};

#[derive(Copy, Clone, Default)]
pub(crate) struct InternedString(&'static str);

impl InternedString {
    pub(crate) fn str(self) -> &'static str {
        self.0
    }
}

pub(crate) fn intern(s: &str) -> InternedString {
    static INTERN: OnceCell<Mutex<Set<&'static str>>> = OnceCell::new();

    let mut set = INTERN
        .get_or_init(|| Mutex::new(Set::new()))
        .lock()
        .unwrap_or_else(PoisonError::into_inner);

    InternedString(match set.get(s) {
        Some(interned) => *interned,
        None => {
            let interned = Box::leak(Box::from(s));
            set.insert(interned);
            interned
        }
    })
}
