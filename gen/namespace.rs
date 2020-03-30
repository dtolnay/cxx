use std::fmt::{self, Display};
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Clone)]
pub struct Namespace {
    segments: Vec<String>,
}

impl Namespace {
    pub fn new(segments: Vec<String>) -> Self {
        Namespace { segments }
    }

    pub fn iter(&self) -> Iter<String> {
        self.segments.iter()
    }
}

impl Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for segment in self {
            f.write_str(segment)?;
            f.write_str("$")?;
        }
        Ok(())
    }
}

impl<'a> IntoIterator for &'a Namespace {
    type Item = &'a String;
    type IntoIter = Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Namespace {
    type Item = String;
    type IntoIter = IntoIter<String>;
    fn into_iter(self) -> Self::IntoIter {
        self.segments.into_iter()
    }
}
