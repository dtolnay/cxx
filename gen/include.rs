use std::fmt::{self, Display};

pub static HEADER: &str = include_str!("include/cxx.h");

pub fn get(guard: &str) -> &'static str {
    let ifndef = format!("#ifndef {}", guard);
    let endif = format!("#endif // {}", guard);
    let begin = find_line(&ifndef);
    let end = find_line(&endif);
    if let (Some(begin), Some(end)) = (begin, end) {
        &HEADER[begin..end + endif.len()]
    } else {
        panic!("not found in cxx.h header: {}", guard)
    }
}

fn find_line(line: &str) -> Option<usize> {
    let mut offset = 0;
    loop {
        offset += HEADER[offset..].find(line)?;
        let rest = &HEADER[offset + line.len()..];
        if rest.starts_with('\n') || rest.starts_with('\r') {
            return Some(offset);
        }
        offset += line.len();
    }
}

#[derive(Default, PartialEq)]
pub struct Includes {
    custom: Vec<String>,
    pub array: bool,
    pub cstdint: bool,
    pub cstring: bool,
    pub exception: bool,
    pub memory: bool,
    pub string: bool,
    pub type_traits: bool,
    pub utility: bool,
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
        if self.array {
            writeln!(f, "#include <array>")?;
        }
        if self.cstdint {
            writeln!(f, "#include <cstdint>")?;
        }
        if self.cstring {
            writeln!(f, "#include <cstring>")?;
        }
        if self.exception {
            writeln!(f, "#include <exception>")?;
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
        if self.utility {
            writeln!(f, "#include <utility>")?;
        }
        if *self != Self::default() {
            writeln!(f)?;
        }
        Ok(())
    }
}
