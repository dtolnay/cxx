use std::io::{self, Write};
use std::panic::{self, UnwindSafe};
use std::process;

pub fn catch_unwind<F, R>(label: &'static str, foreign_call: F) -> R
where
    F: FnOnce() -> R + UnwindSafe,
{
    match panic::catch_unwind(foreign_call) {
        Ok(ret) => ret,
        Err(_) => abort(label),
    }
}

#[cold]
fn abort(label: &'static str) -> ! {
    let mut stdout = io::stdout();
    let _ = writeln!(stdout, "Error: panic in ffi function {}, aborting.", label);
    process::abort();
}
