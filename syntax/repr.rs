use crate::syntax::Atom::{self, *};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{Ident, LitInt};

#[derive(Copy, Clone, PartialEq)]
pub enum Repr {
    Align(u32),
    Atom(Atom),
}

impl Parse for Repr {
    fn parse(input: ParseStream) -> Result<Self> {
        let begin = input.cursor();
        let ident: Ident = input.parse()?;
        if let Some(atom) = Atom::from(&ident) {
            match atom {
                U8 | U16 | U32 | U64 | Usize | I8 | I16 | I32 | I64 | Isize if input.is_empty() => {
                    return Ok(Repr::Atom(atom));
                }
                _ => {}
            }
        } else if ident == "align" {
            let content;
            syn::parenthesized!(content in input);
            let alignment: u32 = content.parse::<LitInt>()?.base10_parse()?;
            if !alignment.is_power_of_two() {
                return Err(Error::new_spanned(
                    begin.token_stream(),
                    "invalid `repr(align)` attribute: not a power of two",
                ));
            }
            if alignment > 2u32.pow(29) {
                return Err(Error::new_spanned(
                    begin.token_stream(),
                    "invalid `repr(align)` attribute: larger than 2^29",
                ));
            }
            return Ok(Repr::Align(alignment));
        }
        Err(Error::new_spanned(
            begin.token_stream(),
            "unrecognized repr",
        ))
    }
}
