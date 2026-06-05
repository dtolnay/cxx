#pragma once

#include <kj/refcount.h>

extern "C" {

// The `rc` inputs point to a Rust `kj_rs::repr::KjRc<T>`, which mirrors `kj::Rc<T>` as two raw
// pointers: first the `kj::Refcounted*` control object, then the `T*` pointee. The helpers below
// only inspect or mutate the control pointer/refcount; they do not dereference the `T*` pointee.

// Returns whether the input `kj::Rc<T>` has more than one outstanding reference.
bool cxxbridge$kjrs$rc$is_shared(const void* rc);

// Constructs a cloned `kj::Rc<T>` into `out`. `out` points to uninitialized storage with the same
// two-pointer layout as `kj::Rc<T>`.
void cxxbridge$kjrs$rc$clone(const void* rc, void* out);

// Destroys the input `kj::Rc<T>` in place, decrementing the control pointer's refcount.
void cxxbridge$kjrs$rc$drop(void* rc);
}
