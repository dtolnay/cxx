use cxx::{let_cxx_string, CxxString};

#[test]
fn test_async_cxx_string() {
    async fn f() {
        let_cxx_string!(s = "...");

        async fn g(_: &CxxString) {}
        g(&s).await;
    }

    // https://github.com/dtolnay/cxx/issues/693
    fn assert_send(_: impl Send) {}
    assert_send(f());
}

#[test]
fn test_cmp_rust_str_cxx_string() {
    use std::cmp::Ordering;
    fn test_eq(rust_str: &str) {
        let_cxx_string!(cxx_string = rust_str);
        assert_eq!(rust_str.partial_cmp(&*cxx_string), Some(Ordering::Equal));
        assert_eq!((*cxx_string).partial_cmp(rust_str), Some(Ordering::Equal));
        assert_eq!(*cxx_string, *rust_str);
        assert_eq!(*rust_str, *cxx_string);
    }

    fn test_ne(lhs: &str, rhs: &str) {
        let_cxx_string!(cxx_lhs = lhs);
        let_cxx_string!(cxx_rhs = rhs);
        assert_ne!(lhs.partial_cmp(&*cxx_rhs), Some(Ordering::Equal));
        assert_ne!(rhs.partial_cmp(&*cxx_lhs), Some(Ordering::Equal));
        assert_ne!((*cxx_lhs).partial_cmp(rhs), Some(Ordering::Equal));
        assert_ne!((*cxx_rhs).partial_cmp(lhs), Some(Ordering::Equal));
        assert_ne!(*cxx_lhs, *rhs);
        assert_ne!(*cxx_rhs, *lhs);
        assert_ne!(*rhs, *cxx_lhs);
        assert_ne!(*lhs, *cxx_rhs);
    }
    test_eq("abc");
    test_eq("");
    test_ne("abc", "Abc");
    test_ne("abc", "");
    // test utf8 character
    test_eq("♡");
}
