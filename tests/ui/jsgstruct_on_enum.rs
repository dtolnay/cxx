#[cxx::bridge]
mod ffi {
    #[derive(JsgStruct)]
    enum MyEnum {
        Variant1,
        Variant2,
    }
}

fn main() {}