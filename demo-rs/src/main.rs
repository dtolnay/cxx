#[cxx::bridge(namespace = org::example)]
mod ffi {
    struct SharedThing {
        z: i32,
        y: Box<ThingR>,
        x: UniquePtr<ThingC>,
    }

    extern "C" {
        include!("demo-cxx/demo.h");

        type ThingC;
        #[alias(snake_case_method)]
        fn camelCaseMethod(&self);
        #[alias(snake_case_function)]
        fn camelCaseFunction();
        #[alias(i32_method)]
        fn overloadedMethod(&self, x: i32);
        #[alias(f32_method)]
        fn overloadedMethod(&self, x: f32);
        #[alias(i32_function)]
        fn overloadedFunction(x: i32);
        #[alias(f32_function)]
        fn overloadedFunction(x: f32);
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn get_name(thing: &ThingC) -> &CxxString;
        fn do_thing(state: SharedThing);
    }

    extern "Rust" {
        type ThingR;
        fn print_r(r: &ThingR);
    }
}

pub struct ThingR(usize);

fn print_r(r: &ThingR) {
    println!("called back with r={}", r.0);
}

fn main() {
    let x = ffi::make_demo("demo of cxx::bridge");
    println!("this is a {}", ffi::get_name(x.as_ref().unwrap()));
    
    x.snake_case_method();
    ffi::snake_case_function();
    x.i32_method(10);
    x.f32_method(10.5);
    ffi::i32_function(10);
    ffi::f32_function(10.5);
    
    ffi::do_thing(ffi::SharedThing {
        z: 222,
        y: Box::new(ThingR(333)),
        x,
    });
}
