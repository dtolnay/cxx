use std::hash::{Hash, Hasher};

#[derive(Copy, Clone)]
pub struct Span(pub proc_macro2::Span);

impl Hash for Span {
    fn hash<H: Hasher>(&self, _state: &mut H) {}
}

impl Eq for Span {}

impl PartialEq for Span {
    fn eq(&self, _other: &Span) -> bool {
        true
    }
}
