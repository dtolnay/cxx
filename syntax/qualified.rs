use syn::parse::{ParseStream, Result};
use syn::{Ident, Path};

pub struct QualifiedName {
    pub segments: Vec<Ident>,
}

impl QualifiedName {
    pub fn parse_unquoted(input: ParseStream) -> Result<Self> {
        let path = input.call(Path::parse_mod_style)?;
        let mut segments = Vec::with_capacity(path.segments.len());
        for segment in path.segments {
            segments.push(segment.ident);
        }
        Ok(QualifiedName { segments })
    }
}
