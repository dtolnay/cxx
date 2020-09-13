#[cxx::bridge(namespace = correct)]
mod here {
    extern "C" {
        type StringPiece;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = folly)]
mod there {
    extern "C" {
        type OtherName = crate::here::StringPiece;
    }
}

fn main() {}
