#[repr(C)]
pub struct FatFunction {
    pub trampoline: *const (),
    pub ptr: *const (),
}
