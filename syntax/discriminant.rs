use crate::syntax::Atom::{self, *};
use proc_macro2::{Literal, Span, TokenStream};
use quote::ToTokens;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::str::FromStr;
use syn::{Error, Expr, Lit, Result, Token, UnOp};

pub struct DiscriminantSet {
    repr: Option<Atom>,
    values: BTreeSet<Discriminant>,
    previous: Option<Discriminant>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Discriminant {
    negative: bool,
    magnitude: u32,
}

impl DiscriminantSet {
    pub fn new(repr: Option<Atom>) -> Self {
        DiscriminantSet {
            repr,
            values: BTreeSet::new(),
            previous: None,
        }
    }

    pub fn insert(&mut self, expr: &Expr) -> Result<Discriminant> {
        let (discriminant, repr) = expr_to_discriminant(expr)?;
        match (self.repr, repr) {
            (None, _) => self.repr = repr,
            (Some(prev), Some(repr)) if prev != repr => {
                let msg = format!("expected {}, found {}", prev, repr);
                return Err(Error::new(Span::call_site(), msg));
            }
            _ => {}
        }
        insert(self, discriminant)
    }

    pub fn insert_next(&mut self) -> Result<Discriminant> {
        let discriminant = match self.previous {
            None => Discriminant::zero(),
            Some(mut discriminant) if discriminant.negative => {
                discriminant.magnitude -= 1;
                if discriminant.magnitude == 0 {
                    discriminant.negative = false;
                }
                discriminant
            }
            Some(mut discriminant) => {
                if discriminant.magnitude == u32::MAX {
                    let msg = format!("discriminant overflow on value after {}", u32::MAX);
                    return Err(Error::new(Span::call_site(), msg));
                }
                discriminant.magnitude += 1;
                discriminant
            }
        };
        insert(self, discriminant)
    }

    pub fn inferred_repr(&self) -> Result<Atom> {
        if let Some(repr) = self.repr {
            return Ok(repr);
        }
        if self.values.is_empty() {
            return Ok(U8);
        }
        let min = *self.values.iter().next().unwrap();
        let max = *self.values.iter().next_back().unwrap();
        for bounds in &BOUNDS {
            if bounds.min <= min && max <= bounds.max {
                return Ok(bounds.repr);
            }
        }
        let msg = "these discriminant values do not fit in any supported enum repr type";
        Err(Error::new(Span::call_site(), msg))
    }
}

fn expr_to_discriminant(expr: &Expr) -> Result<(Discriminant, Option<Atom>)> {
    match expr {
        Expr::Lit(expr) => {
            if let Lit::Int(lit) = &expr.lit {
                let discriminant = lit.base10_parse::<Discriminant>()?;
                let repr = parse_int_suffix(lit.suffix())?;
                return Ok((discriminant, repr));
            }
        }
        Expr::Unary(unary) => {
            if let UnOp::Neg(_) = unary.op {
                let (mut discriminant, repr) = expr_to_discriminant(&unary.expr)?;
                discriminant.negative ^= true;
                return Ok((discriminant, repr));
            }
        }
        _ => {}
    }
    Err(Error::new_spanned(
        expr,
        "enums with non-integer literal discriminants are not supported yet",
    ))
}

fn insert(set: &mut DiscriminantSet, discriminant: Discriminant) -> Result<Discriminant> {
    if let Some(expected_repr) = set.repr {
        for bounds in &BOUNDS {
            if bounds.repr != expected_repr {
                continue;
            }
            if bounds.min <= discriminant && discriminant <= bounds.max {
                break;
            }
            let msg = format!(
                "discriminant value `{}` is outside the limits of {}",
                discriminant, expected_repr,
            );
            return Err(Error::new(Span::call_site(), msg));
        }
    }
    if set.values.insert(discriminant) {
        set.previous = Some(discriminant);
        Ok(discriminant)
    } else {
        let msg = format!("discriminant value `{}` already exists", discriminant);
        Err(Error::new(Span::call_site(), msg))
    }
}

impl Discriminant {
    const fn zero() -> Self {
        Discriminant {
            negative: false,
            magnitude: 0,
        }
    }

    const fn pos(u: u32) -> Self {
        Discriminant {
            negative: false,
            magnitude: u,
        }
    }

    const fn neg(i: i32) -> Self {
        Discriminant {
            negative: i < 0,
            // This is `i.abs() as u32` but without overflow on MIN. Uses the
            // fact that MIN.wrapping_abs() wraps back to MIN whose binary
            // representation is 1<<31, and thus the `as u32` conversion
            // produces 1<<31 too which happens to be the correct unsigned
            // magnitude.
            magnitude: i.wrapping_abs() as u32,
        }
    }
}

impl Display for Discriminant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.negative {
            f.write_str("-")?;
        }
        Display::fmt(&self.magnitude, f)
    }
}

impl ToTokens for Discriminant {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.negative {
            Token![-](Span::call_site()).to_tokens(tokens);
        }
        Literal::u32_unsuffixed(self.magnitude).to_tokens(tokens);
    }
}

impl FromStr for Discriminant {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self> {
        let negative = s.starts_with('-');
        if negative {
            s = &s[1..];
        }
        match s.parse::<u32>() {
            Ok(magnitude) => Ok(Discriminant {
                negative,
                magnitude,
            }),
            Err(_) => Err(Error::new(
                Span::call_site(),
                "discriminant value outside of supported range",
            )),
        }
    }
}

impl Ord for Discriminant {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (true, true) => self.magnitude.cmp(&other.magnitude).reverse(),
            (true, false) => Ordering::Less, // negative < positive
            (false, true) => Ordering::Greater, // positive > negative
            (false, false) => self.magnitude.cmp(&other.magnitude),
        }
    }
}

impl PartialOrd for Discriminant {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn parse_int_suffix(suffix: &str) -> Result<Option<Atom>> {
    if suffix.is_empty() {
        return Ok(None);
    }
    if let Some(atom) = Atom::from_str(suffix) {
        match atom {
            U8 | U16 | U32 | U64 | Usize | I8 | I16 | I32 | I64 | Isize => return Ok(Some(atom)),
            _ => {}
        }
    }
    let msg = format!("unrecognized integer suffix: `{}`", suffix);
    Err(Error::new(Span::call_site(), msg))
}

struct Bounds {
    repr: Atom,
    min: Discriminant,
    max: Discriminant,
}

const BOUNDS: [Bounds; 6] = [
    Bounds {
        repr: U8,
        min: Discriminant::zero(),
        max: Discriminant::pos(u8::MAX as u32),
    },
    Bounds {
        repr: I8,
        min: Discriminant::neg(i8::MIN as i32),
        max: Discriminant::pos(i8::MAX as u32),
    },
    Bounds {
        repr: U16,
        min: Discriminant::zero(),
        max: Discriminant::pos(u16::MAX as u32),
    },
    Bounds {
        repr: I16,
        min: Discriminant::neg(i16::MIN as i32),
        max: Discriminant::pos(i16::MAX as u32),
    },
    Bounds {
        repr: U32,
        min: Discriminant::zero(),
        max: Discriminant::pos(u32::MAX),
    },
    Bounds {
        repr: I32,
        min: Discriminant::neg(i32::MIN),
        max: Discriminant::pos(i32::MAX as u32),
    },
];
