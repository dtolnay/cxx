use crate::ffi;
use kj_rs::{KjArc, KjRc};

pub fn modify_own_ret_rc(
    mut rc: KjRc<ffi::OpaqueRefcountedClass>,
) -> KjRc<ffi::OpaqueRefcountedClass> {
    let mut_ref = rc.get_mut();
    mut_ref.unwrap().pin_mut().set_data(467);
    rc
}

pub fn modify_own_ret_arc(
    mut arc: KjArc<ffi::OpaqueAtomicRefcountedClass>,
) -> KjArc<ffi::OpaqueAtomicRefcountedClass> {
    let mut_ref = arc.get_mut();
    mut_ref.unwrap().pin_mut().set_data(328);
    arc
}

#[cfg(test)]
pub mod tests {
    use kj_rs::refcount::{AtomicRefcounted, Refcounted};

    use crate::ffi;

    #[test]
    fn test_rc() {
        let rc = ffi::get_rc();
        assert_eq!(rc.get_data(), 15);
        let rc_clone = rc.clone();
        assert_eq!(rc_clone.get_data(), 15);

        assert!(rc.is_shared());
        std::mem::drop(rc_clone);
        assert!(!rc.is_shared());
    }

    #[test]
    fn test_arc() {
        let arc = ffi::get_arc();
        assert_eq!(arc.get_data(), 16);
        let arc_clone = arc.clone();
        assert_eq!(arc_clone.get_data(), 16);

        assert!(arc.is_shared());
        std::mem::drop(arc_clone);
        assert!(!arc.is_shared());
    }

    #[test]
    fn test_pass_cxx() {
        let rc = ffi::get_rc();
        let arc = ffi::get_arc();
        ffi::give_rc_back(rc);
        ffi::give_arc_back(arc);
    }

    #[test]
    fn test_arc_thread() {
        let mut arc = ffi::get_arc();

        std::thread::scope(|s| {
            let mut c = arc.clone();
            assert!(c.get_mut().is_none());

            s.spawn(|| {
                assert_eq!(arc.clone().get_data(), 16);
            });

            s.spawn(|| {
                let a = arc.clone();
                assert!(a.is_shared());
            });
        });

        {
            let mut cloned = arc.clone();
            assert!(cloned.get_mut().is_none());
        }

        let mut_ref = arc.get_mut();
        assert!(mut_ref.is_some());
        mut_ref.unwrap().pin_mut().set_data(35643);
        assert_eq!(arc.get_data(), 35643);
    }
}
