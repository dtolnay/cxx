pub struct True;
pub struct False;

pub trait ToBool {
    type Bool: Sized;
    const BOOL: Self::Bool;
}

impl ToBool for [(); 0] {
    type Bool = False;
    const BOOL: Self::Bool = False;
}

impl ToBool for [(); 1] {
    type Bool = True;
    const BOOL: Self::Bool = True;
}

macro_rules! bool {
    ($e:expr) => {{
        const EXPR: bool = $e;
        <[(); EXPR as usize] as $crate::assert::ToBool>::BOOL
    }};
}

macro_rules! const_assert {
    ($e:expr) => {
        const _: $crate::assert::True = bool!($e);
    };
}
