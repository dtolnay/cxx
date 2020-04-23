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
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn get_name(self: &ThingC) -> &CxxString;
        fn do_thing(state: SharedThing);
    }

    extern "Rust" {
        type ThingR;
        fn print(&self);
    }
}

pub struct ThingR(usize);

impl ThingR {
    fn print(&self) {
        println!("called back with r={}", self.0);
    }
}

fn main() {
    let x = ffi::make_demo("demo of cxx::bridge");
    println!("this is a {}", x.get_name());

    ffi::do_thing(ffi::SharedThing {
        z: 222,
        y: Box::new(ThingR(333)),
        x,
    });
}
