#include "refcount.h"

#include <new>

extern "C" {

// `kj::Rc<T>` operations used by Rust (`isShared()`, `addRef()`, destructor) operate only on the
// `Refcounted*` control pointer stored in the first word of the `kj::Rc` object. They do not depend
// on, cast, or dereference the second word (`T*`). Therefore a `kj::Rc<T>` with any T can be treated
// here as a `kj::Rc<kj::Refcounted>` for refcount-only operations.

bool cxxbridge$kjrs$rc$is_shared(const void* rc) {
  auto refcounted = *reinterpret_cast<kj::Refcounted* const*>(rc);
  return refcounted->isShared();
}

void cxxbridge$kjrs$rc$clone(const void* rc, void* out) {
  auto typed =
      const_cast<kj::Rc<kj::Refcounted>*>(reinterpret_cast<const kj::Rc<kj::Refcounted>*>(rc));
  ::new (out) kj::Rc<kj::Refcounted>(typed->addRef());
}

void cxxbridge$kjrs$rc$drop(void* rc) {
  reinterpret_cast<kj::Rc<kj::Refcounted>*>(rc)->~Rc();
}
}
