use crate::syntax::Atom::{self, *};
use proc_macro2::{Literal, Span, TokenStream};
use quote::ToTokens;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt::{self, Display};
use std::str::FromStr;
use std::u64;
use syn::{Error, Expr, Lit, Result, Token, UnOp};

pub struct DiscriminantSet {
    repr: Option<Atom>,
    values: BTreeSet<Discriminant>,
    previous: Option<Discriminant>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Discriminant {
    negative: bool,
    magnitude: u64,
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
            (None, Some(new_repr)) => {
                if let Some(limits) = Limits::of(new_repr) {
                    for &past in &self.values {
                        if limits.min <= past && past <= limits.max {
                            continue;
                        }
                        let msg = format!(
                            "discriminant value `{}` is outside the limits of {}",
                            past, new_repr,
                        );
                        return Err(Error::new(Span::call_site(), msg));
                    }
                }
                self.repr = Some(new_repr);
            }
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
                if discriminant.magnitude == u64::MAX {
                    let msg = format!("discriminant overflow on value after {}", u64::MAX);
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
        for limits in &LIMITS {
            if limits.min <= min && max <= limits.max {
                return Ok(limits.repr);
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
        if let Some(limits) = Limits::of(expected_repr) {
            if discriminant < limits.min || limits.max < discriminant {
                let msg = format!(
                    "discriminant value `{}` is outside the limits of {}",
                    discriminant, expected_repr,
                );
                return Err(Error::new(Span::call_site(), msg));
            }
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

    const fn pos(u: u64) -> Self {
        Discriminant {
            negative: false,
            magnitude: u,
        }
    }

    const fn neg(i: i64) -> Self {
        Discriminant {
            negative: i < 0,
            // This is `i.abs() as u64` but without overflow on MIN. Uses the
            // fact that MIN.wrapping_abs() wraps back to MIN whose binary
            // representation is 1<<63, and thus the `as u64` conversion
            // produces 1<<63 too which happens to be the correct unsigned
            // magnitude.
            magnitude: i.wrapping_abs() as u64,
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
        Literal::u64_unsuffixed(self.magnitude).to_tokens(tokens);
    }
}

impl FromStr for Discriminant {
    type Err = Error;

    fn from_str(mut s: &str) -> Result<Self> {
        let negative = s.starts_with('-');
        if negative {
            s = &s[1..];
        }
        match s.parse::<u64>() {
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

#[derive(Copy, Clone)]
struct Limits {
    repr: Atom,
    min: Discriminant,
    max: Discriminant,
}

impl Limits {
    fn of(repr: Atom) -> Option<Limits> {
        for limits in &LIMITS {
            if limits.repr == repr {
                return Some(*limits);
            }
        }
        None
    }
}

const LIMITS: [Limits; 8] = [
    Limits {
        repr: U8,
        min: Discriminant::zero(),
        max: Discriminant::pos(std::u8::MAX as u64),
    },
    Limits {
        repr: I8,
        min: Discriminant::neg(std::i8::MIN as i64),
        max: Discriminant::pos(std::i8::MAX as u64),
    },
    Limits {
        repr: U16,
        min: Discriminant::zero(),
        max: Discriminant::pos(std::u16::MAX as u64),
    },
    Limits {
        repr: I16,
        min: Discriminant::neg(std::i16::MIN as i64),
        max: Discriminant::pos(std::i16::MAX as u64),
    },
    Limits {
        repr: U32,
        min: Discriminant::zero(),
        max: Discriminant::pos(std::u32::MAX as u64),
    },
    Limits {
        repr: I32,
        min: Discriminant::neg(std::i32::MIN as i64),
        max: Discriminant::pos(std::i32::MAX as u64),
    },
    Limits {
        repr: U64,
        min: Discriminant::zero(),
        max: Discriminant::pos(std::u64::MAX),
    },
    Limits {
        repr: I64,
        min: Discriminant::neg(std::i64::MIN),
        max: Discriminant::pos(std::i64::MAX as u64),
    },
];
