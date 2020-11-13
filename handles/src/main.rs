mod handle;

use cxx::CxxString;

#[cxx::bridge(namespace = my_usage::ffi)]
mod ffi {
    #[namespace = "zx::ffi"]
    extern "C++" {
        include!("cxx-handles-demo/src/handle.rs.h");
        type Job = crate::handle::ffi::Job;
        type Process = crate::handle::ffi::Process;
    }

    extern "Rust" {
        fn create_process(name: &CxxString, job: &Job) -> Process;
    }
}

fn create_process(name: &CxxString, job: &handle::ffi::Job) -> handle::ffi::Process {
    println!("{:?} job={}", name, job.raw);
    handle::ffi::Process { raw: 0 }
}

fn main() {}
