use std::fmt::{self, Display};

pub static HEADER: &str = include_str!("include/cxx.h");

pub fn get(guard: &str) -> &'static str {
    let ifndef = format!("#ifndef {}", guard);
    let endif = format!("#endif // {}", guard);
    let begin = HEADER.find(&ifndef);
    let end = HEADER.find(&endif);
    if let (Some(begin), Some(end)) = (begin, end) {
        &HEADER[begin..end + endif.len()]
    } else {
        panic!("not found in cxx.h header: {}", guard)
    }
}

#[derive(Default)]
pub struct Includes {
    custom: Vec<String>,
    pub cstdint: bool,
    pub memory: bool,
    pub string: bool,
    pub type_traits: bool,
}

impl Includes {
    pub fn new() -> Self {
        Includes::default()
    }

    pub fn insert(&mut self, include: String) {
        self.custom.push(include);
    }
}

impl Display for Includes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for include in &self.custom {
            writeln!(f, "#include \"{}\"", include.escape_default())?;
        }
        if self.cstdint {
            writeln!(f, "#include <cstdint>")?;
        }
        if self.memory {
            writeln!(f, "#include <memory>")?;
        }
        if self.string {
            writeln!(f, "#include <string>")?;
        }
        if self.type_traits {
            writeln!(f, "#include <type_traits>")?;
        }
        Ok(())
    }
}
