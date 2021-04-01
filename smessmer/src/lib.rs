#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        #[derive(ExternType)]
        type MyCustomType;

        fn rustfunc(v: &MyCustomType);
    }
}

pub struct MyCustomType {}

fn rustfunc(v: &MyCustomType) {
    let _ = v;
    println!("success");
}
