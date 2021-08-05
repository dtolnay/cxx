use cxx::{let_cxx_vector, CxxVector};

#[test]
fn test_cxx_vector_infer() {
    let_cxx_vector!(v);

    v.as_mut().push(12usize);
    v.as_mut().push(13);

    assert_eq!(v.len(), 2);
}

#[test]
fn test_cxx_vector_ascribed() {
    let_cxx_vector!(v: CxxVector<i32>);

    v.as_mut().push(12);
    v.as_mut().push(13);

    assert_eq!(v.len(), 2);
}

#[test]
fn test_cxx_vector_lit() {
    let_cxx_vector!(v = [1, 2, 3,]);

    assert_eq!(v.len(), 3);
}

#[test]
fn test_cxx_vector_lit_ascribed() {
    let_cxx_vector!(v: CxxVector<usize> = [1, 2, 3, 4, 5,]);

    assert_eq!(v.iter().sum::<usize>(), 15);
}
