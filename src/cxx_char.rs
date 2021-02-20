use std::char::CharTryFromError;
use std::convert::{From, TryFrom};

/// Binding to C++ `char32_t`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct CxxChar(u32);

impl TryFrom<CxxChar> for char {
    type Error = CharTryFromError;

    fn try_from(value: CxxChar) -> Result<Self, Self::Error> {
        char::try_from(value.0)
    }
}

impl From<char> for CxxChar {
    fn from(ch: char) -> Self {
        CxxChar(ch as u32)
    }
}
