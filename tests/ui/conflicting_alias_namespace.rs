#[cxx::bridge(namespace = correct)]
mod here {
    extern "C" {
        type StringPiece;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = other)]
#[cxx::alias_namespace(crate::here = correct)]
#[cxx::alias_namespace(crate::here = folly)]
mod there {
    extern "C" {
        type StringPiece = crate::here::StringPiece;
    }
}

fn main() {}
