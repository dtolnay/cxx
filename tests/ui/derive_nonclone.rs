#[cxx::bridge]
mod ffi {
    #[derive(Clone)]
    struct TryClone {
        other: Other,
    }

    struct Other {
        x: usize,
    }
}

fn main() {}
