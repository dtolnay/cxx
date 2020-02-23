use cxx_test_suite::ffi;
use std::cell::Cell;
use std::ffi::CStr;

thread_local! {
    static CORRECT: Cell<bool> = Cell::new(false);
}

#[no_mangle]
extern "C" fn cxx_test_suite_set_correct() {
    CORRECT.with(|correct| correct.set(true));
}

macro_rules! check {
    ($run:expr) => {{
        CORRECT.with(|correct| correct.set(false));
        $run;
        assert!(CORRECT.with(|correct| correct.get()), stringify!($run));
    }};
}

#[test]
fn test_c_return() {
    let shared = ffi::Shared { z: 2020 };

    assert_eq!(2020, ffi::c_return_primitive());
    assert_eq!(2020, ffi::c_return_shared().z);
    ffi::c_return_unique_ptr();
    assert_eq!(2020, *ffi::c_return_ref(&shared));
    assert_eq!("2020", ffi::c_return_str(&shared));
    assert_eq!("2020", ffi::c_return_rust_string());
    assert_eq!(
        "2020",
        ffi::c_return_unique_ptr_string()
            .as_ref()
            .unwrap()
            .to_str()
            .unwrap()
    );
}

#[test]
fn test_c_take() {
    let unique_ptr = ffi::c_return_unique_ptr();

    check!(ffi::c_take_primitive(2020));
    check!(ffi::c_take_shared(ffi::Shared { z: 2020 }));
    check!(ffi::c_take_box(Box::new(())));
    check!(ffi::c_take_ref_c(unique_ptr.as_ref().unwrap()));
    check!(ffi::c_take_unique_ptr(unique_ptr));
    check!(ffi::c_take_str("2020"));
    check!(ffi::c_take_rust_string("2020".to_owned()));
    check!(ffi::c_take_unique_ptr_string(
        ffi::c_return_unique_ptr_string()
    ));
}

#[test]
fn test_c_call_r() {
    fn cxx_run_test() {
        extern "C" {
            fn cxx_run_test() -> *const i8;
        }
        let failure = unsafe { cxx_run_test() };
        if !failure.is_null() {
            let msg = unsafe { CStr::from_ptr(failure) };
            eprintln!("{}", msg.to_string_lossy());
        }
    }
    check!(cxx_run_test());
}
