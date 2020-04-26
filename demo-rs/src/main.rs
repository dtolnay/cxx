#[cxx::bridge(namespace = org::example)]
mod ffi {
    struct SharedThing {
        z: i32,
        y: Box<ThingR>,
        x: UniquePtr<ThingC>,
    }

    struct JsonBlob {
        json: UniquePtr<CxxString>,
        blob: UniquePtr<Vector<u8>>,
    }

    extern "C" {
        include!("demo-cxx/demo.h");

        type ThingC;
        fn make_demo(appname: &str) -> UniquePtr<ThingC>;
        fn get_name(thing: &ThingC) -> &CxxString;
        fn do_thing(state: SharedThing) -> UniquePtr<Vector<u8>>;
        fn get_jb(v: &Vec<u8>) -> JsonBlob;
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

    let vec = ffi::do_thing(ffi::SharedThing {
        z: 222,
        y: Box::new(ThingR(333)),
        x,
    });

    println!("vec length = {}", vec.as_ref().unwrap().size());
    for (i, v) in vec.as_ref().unwrap().into_iter().enumerate() {
        println!("vec[{}] = {}", i, v);
    }

    let mut rv: Vec<u8> = Vec::new();
    for _ in 0..1000 {
        rv.push(33);
    }
    let jb = ffi::get_jb(&rv);
    println!("json: {}", jb.json.as_ref().unwrap());
    for (i, v) in jb.blob.as_ref().unwrap().into_iter().enumerate() {
        println!("jb.blob[{}] = {}", i, v);
    }
}
