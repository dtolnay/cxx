#[derive(Default, PartialEq)]
pub struct Builtins {
    pub panic: bool,
    pub rust_string: bool,
    pub rust_str: bool,
    pub rust_slice: bool,
    pub rust_box: bool,
    pub rust_vec: bool,
    pub rust_fn: bool,
    pub rust_isize: bool,
    pub unsafe_bitcopy: bool,
    pub rust_error: bool,
    pub manually_drop: bool,
    pub maybe_uninit: bool,
    pub trycatch: bool,
    pub rust_str_new_unchecked: bool,
    pub rust_str_repr: bool,
}

impl Builtins {
    pub fn new() -> Self {
        Builtins::default()
    }
}
