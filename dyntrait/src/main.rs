#![no_main]

use anyhow::Result;
use cxx::ExternType;

pub trait MyData {
    fn traitfn(&self);
}

unsafe impl ExternType for Box<dyn MyData> {
    type Id = cxx::type_id!("BoxDynMyData");
    type Kind = cxx::kind::Trivial;
}

#[repr(transparent)]
pub struct PtrBoxDynMyData(*mut Box<dyn MyData>);
unsafe impl ExternType for PtrBoxDynMyData {
    type Id = cxx::type_id!("PtrBoxDynMyData");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("demo-dyntrait/include/mydata.h");
        type BoxDynMyData = Box<dyn crate::MyData>;
        type PtrBoxDynMyData = crate::PtrBoxDynMyData;
    }

    extern "Rust" {
        fn dyn_mydata_traitfn(mydata: &BoxDynMyData);
        unsafe fn dyn_mydata_drop_in_place(ptr: PtrBoxDynMyData);

        fn read_data() -> Result<BoxDynMyData>;
    }
}

fn dyn_mydata_traitfn(mydata: &Box<dyn MyData>) {
    (**mydata).traitfn();
}

unsafe fn dyn_mydata_drop_in_place(ptr: PtrBoxDynMyData) {
    std::ptr::drop_in_place(ptr.0);
}

fn read_data() -> Result<Box<dyn MyData>> {
    struct Implementation(usize);
    impl MyData for Implementation {
        fn traitfn(&self) {
            println!("it worked! {}", self.0);
        }
    }
    Ok(Box::new(Implementation(9)))
}
