#include "test-refcount.h"

#include "kj-rs-demo/lib.rs.h"
#include "kj-rs/tests/lib.rs.h"
#include "kj/common.h"
#include "kj/debug.h"

namespace kj_rs_demo {
kj::Rc<OpaqueRefcountedClass> get_rc() {
  return kj::rc<OpaqueRefcountedClass>(15);
}

kj::Arc<OpaqueAtomicRefcountedClass> get_arc() {
  return kj::arc<OpaqueAtomicRefcountedClass>(16);
}
void give_arc_back(kj::Arc<OpaqueAtomicRefcountedClass> arc) {
  kj::Arc<OpaqueAtomicRefcountedClass> ret_arc = modify_own_ret_arc(kj::mv(arc));
  KJ_ASSERT(ret_arc->getData() == 328);
}
void give_rc_back(kj::Rc<OpaqueRefcountedClass> rc) {
  kj::Rc<OpaqueRefcountedClass> ret_rc = modify_own_ret_rc(kj::mv(rc));
  KJ_ASSERT(ret_rc->getData() == 467);
}
}  // namespace kj_rs_demo
