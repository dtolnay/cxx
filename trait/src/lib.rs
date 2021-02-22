#![allow(non_camel_case_types)]

extern crate alloc;

mod extern_type;
mod type_id;

pub use crate::chars::*;
pub use crate::extern_type::{kind, ExternType};

// Not public API.
#[doc(hidden)]
pub mod private {
    pub use cxxbridge_macro::type_id;
}

macro_rules! chars {
    ($($ch:ident)*) => {
        $(
            #[doc(hidden)]
            pub enum $ch {}
        )*
    };
}

pub mod chars {
    chars! {
        _0 _1 _2 _3 _4 _5 _6 _7 _8 _9
        A B C D E F G H I J K L M N O P Q R S T U V W X Y Z
        a b c d e f g h i j k l m n o p q r s t u v w x y z
        __ // underscore
    }
}
