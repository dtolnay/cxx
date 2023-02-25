pub mod other_module {
    pub struct Source {
        member: u32,
    }
}

#[cxx::bridge]
mod ffi {
    extern "Rust" {
        // Not allowed - the target is not `extern "Rust"`.
        type Source = crate::other_module::Source;
    }
}

fn main() {}
