use crate::syntax::Atom::{self, *};
use proc_macro2::Ident;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{parenthesized, LitInt};

pub(crate) enum Repr {
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
            parenthesized!(content in input);
            let align_lit: LitInt = content.parse()?;
            let align: u32 = align_lit.base10_parse()?;
            if !align.is_power_of_two() {
                return Err(Error::new_spanned(
                    align_lit,
                    "invalid repr(align) attribute: not a power of two",
                ));
            }
            if align > 2u32.pow(29) {
                return Err(Error::new_spanned(
                    align_lit,
                    "invalid repr(align) attribute: larger than 2^29",
                ));
            }
            return Ok(Repr::Align(align));
        }
        Err(Error::new_spanned(
            begin.token_stream(),
            "unrecognized repr",
        ))
    }
}
