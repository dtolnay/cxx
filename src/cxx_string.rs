use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::slice;
use std::str::{self, Utf8Error};

extern "C" {
    #[link_name = "cxxbridge03$cxx_string$data"]
    fn string_data(_: &CxxString) -> *const u8;
    #[link_name = "cxxbridge03$cxx_string$length"]
    fn string_length(_: &CxxString) -> usize;
}

/// Binding to C++ `std::string`.
///
/// # Invariants
///
/// As an invariant of this API and the static analysis of the cxx::bridge
/// macro, in Rust code we can never obtain a `CxxString` by value. C++'s string
/// requires a move constructor and may hold internal pointers, which is not
/// compatible with Rust's move behavior. Instead in Rust code we will only ever
/// look at a CxxString through a reference or smart pointer, as in `&CxxString`
/// or `UniquePtr<CxxString>`.
#[repr(C)]
pub struct CxxString {
    _private: [u8; 0],
}

impl CxxString {
    /// Returns the length of the string in bytes.
    ///
    /// Matches the behavior of C++ [std::string::size][size].
    ///
    /// [size]: https://en.cppreference.com/w/cpp/string/basic_string/size
    pub fn len(&self) -> usize {
        unsafe { string_length(self) }
    }

    /// Returns true if `self` has a length of zero bytes.
    ///
    /// Matches the behavior of C++ [std::string::empty][empty].
    ///
    /// [empty]: https://en.cppreference.com/w/cpp/string/basic_string/empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns a byte slice of this string's contents.
    pub fn as_bytes(&self) -> &[u8] {
        let data = self.as_ptr();
        let len = self.len();
        unsafe { slice::from_raw_parts(data, len) }
    }

    /// Produces a pointer to the first character of the string.
    ///
    /// Matches the behavior of C++ [std::string::data][data].
    ///
    /// Note that the return type may look like `const char *` but is not a
    /// `const char *` in the typical C sense, as C++ strings may contain
    /// internal null bytes. As such, the returned pointer only makes sense as a
    /// string in combination with the length returned by [`len()`][len].
    ///
    /// [data]: https://en.cppreference.com/w/cpp/string/basic_string/data
    /// [len]: #method.len
    pub fn as_ptr(&self) -> *const u8 {
        unsafe { string_data(self) }
    }

    /// Validates that the C++ string contains UTF-8 data and produces a view of
    /// it as a Rust &amp;str, otherwise an error.
    pub fn to_str(&self) -> Result<&str, Utf8Error> {
        str::from_utf8(self.as_bytes())
    }

    /// If the contents of the C++ string are valid UTF-8, this function returns
    /// a view as a Cow::Borrowed &amp;str. Otherwise replaces any invalid UTF-8
    /// sequences with the U+FFFD [replacement character] and returns a
    /// Cow::Owned String.
    ///
    /// [replacement character]: https://doc.rust-lang.org/std/char/constant.REPLACEMENT_CHARACTER.html
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

impl Display for CxxString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(self.to_string_lossy().as_ref(), f)
    }
}

impl Debug for CxxString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self.to_string_lossy().as_ref(), f)
    }
}

impl PartialEq for CxxString {
    fn eq(&self, other: &CxxString) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<CxxString> for str {
    fn eq(&self, other: &CxxString) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<str> for CxxString {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}
