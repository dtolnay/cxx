use anyhow::{anyhow, Result};

#[cxx::bridge]
mod ffi {
    extern "C" {
        include!("no-causes/include/lib.h");

        fn cthing() -> Result<()>;
    }

    extern "Rust" {
        fn rustthing() -> Result<()>;
    }
}

fn rustthing() -> Result<()> {
    Err(anyhow!("no such file or directory").context("failed to open: /nonexist"))
}

fn main() -> Result<()> {
    ffi::cthing()?;
    Ok(())
}
