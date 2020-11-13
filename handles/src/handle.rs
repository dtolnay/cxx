#[cxx::bridge(namespace = zx::ffi)]
pub mod ffi {
    struct Process {
        raw: u32,
    }

    struct Job {
        raw: u32,
    }
}
