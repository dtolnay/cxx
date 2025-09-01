use crate::syntax::symbol::Symbol;
use crate::syntax::Pair;
use std::fmt::{self, Display};

pub(crate) struct Guard {
    kind: &'static str,
    symbol: Symbol,
}

impl Guard {
    pub fn new(kind: &'static str, name: &Pair) -> Self {
        Guard {
            kind,
            symbol: name.to_symbol(),
        }
    }
}

impl Display for Guard {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}_{}", self.kind, self.symbol)
    }
}
