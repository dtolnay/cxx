mod arg {
    use cxx::ExternType;
    use std::marker::PhantomPinned;

    struct Arg(PhantomPinned);

    unsafe impl ExternType for Arg {
        type Id = cxx::type_id!("Arg");
        type Kind = cxx::kind::Opaque;
    }

    #[cxx::bridge]
    mod ffi {
        unsafe extern "C++" {
            type Arg = crate::arg::Arg;
            fn f(arg: &mut Arg);
        }
    }
}

mod receiver {
    use cxx::ExternType;
    use std::marker::PhantomPinned;

    struct Receiver(PhantomPinned);

    unsafe impl ExternType for Receiver {
        type Id = cxx::type_id!("Receiver");
        type Kind = cxx::kind::Opaque;
    }

    #[cxx::bridge]
    mod ffi {
        unsafe extern "C++" {
            type Receiver = crate::receiver::Receiver;
            fn g(&mut self);
        }
    }
}

mod receiver2 {
    use cxx::ExternType;
    use std::marker::PhantomPinned;

    struct Receiver2(PhantomPinned);

    unsafe impl ExternType for Receiver2 {
        type Id = cxx::type_id!("Receiver2");
        type Kind = cxx::kind::Opaque;
    }

    #[cxx::bridge]
    mod ffi {
        unsafe extern "C++" {
            type Receiver2 = crate::receiver2::Receiver2;
            fn h(self: &mut Receiver2);
        }
    }
}

fn main() {}
