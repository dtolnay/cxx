use cxx::{let_cxx_vector, CxxVector};

#[test]
fn test_cxx_vector() {
    let_cxx_vector!(v: CxxVector<i32>);

    v.as_mut().push(12);
    v.as_mut().push(13);

    assert_eq!(v.len(), 2);
}

#[test]
fn test_cxx_vector_infer() {
    let_cxx_vector!(v);

    v.as_mut().push(12usize);
    v.as_mut().push(13);

    assert_eq!(v.len(), 2);
}
