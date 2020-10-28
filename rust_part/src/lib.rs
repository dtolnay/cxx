#[cxx::bridge(namespace = rust_part)]
mod ffi {

    struct SharedThing {
        points: Box<Points>,
        persons: UniquePtr<Person>,
    }

    extern "C++" {
        include!("cpp_part.h");
        type Person;

        fn get_name(person: &Person) -> &CxxString;
        fn make_person() -> UniquePtr<Person>;
    }

    extern "Rust" {
        type Points;
        fn print_shared_thing(points: &SharedThing);
        fn make_shared_thing() -> SharedThing;
        fn rust_echo(val: i32) -> i32;
    }
}

#[derive(Debug)]
pub struct Points {
    x: Vec<u8>,
    y: Vec<u8>,
}

fn print_shared_thing(thing: &ffi::SharedThing) {
    println!("{:#?}", thing.points);
    println!("{:#?}", ffi::get_name(thing.persons.as_ref().unwrap()));
}

fn make_shared_thing() -> ffi::SharedThing {
    ffi::SharedThing {
        points: Box::new(Points {
            x: vec![1, 2, 3],
            y: vec![4, 5, 6],
        }),
        persons: ffi::make_person(),
    }
}

#[inline(always)]
fn rust_echo(val: i32) -> i32 {
    val
}
