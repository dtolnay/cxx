// Aliases of aliases don't work if the namespace of any intermediate alias
// differs from the original bridge's namespace. This is because the ExternType
// trait being checked against will use the original namespace, which the final
// alias does not know. However, if not for the ExternType check, this would
// work because of the C++ "using" aliases generated from each bridge.
//
// This can be made to work if needed - either through an attribute on the alias
// as a "manual" fix, or potentially through some creativity with ExternType
// (like defining a related type that implements ExternType and checking against
// that instead of the actual type) - but seems like a rare case.

#[cxx::bridge(namespace = here)]
mod here {
    extern "C" {
        type C;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = there)]
#[cxx::alias_namespace(crate::here = here)]
mod there {
    extern "C" {
        type C = crate::here::C;
    }
}

// Rustfmt mangles the extern type alias.
// https://github.com/rust-lang/rustfmt/issues/4159
#[rustfmt::skip]
#[cxx::bridge(namespace = anywhere)]
#[cxx::alias_namespace(crate::there = there)]
mod anywhere {
    extern "C" {
        type C = crate::there::C;
    }
}

fn main() {}
