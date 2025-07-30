#pragma once

#include "kj/refcount.h"

#include <cstdint>

namespace kj_rs_demo {

class OpaqueRefcountedClass: public kj::Refcounted {
 public:
  OpaqueRefcountedClass(uint64_t data): data(data) {}
  ~OpaqueRefcountedClass() {}
  uint64_t getData() const {
    return this->data;
  }
  void setData(uint64_t val) {
    this->data = val;
  }

 private:
  uint64_t data;
};

class OpaqueAtomicRefcountedClass: public kj::AtomicRefcounted {
 public:
  OpaqueAtomicRefcountedClass(uint64_t data): data(data) {}
  ~OpaqueAtomicRefcountedClass() {}
  uint64_t getData() const {
    return this->data;
  }
  void setData(uint64_t val) {
    this->data = val;
  }

 private:
  uint64_t data;
};

kj::Rc<OpaqueRefcountedClass> get_rc();
kj::Arc<OpaqueAtomicRefcountedClass> get_arc();

void give_arc_back(kj::Arc<OpaqueAtomicRefcountedClass> arc);
void give_rc_back(kj::Rc<OpaqueRefcountedClass> rc);

}  // namespace kj_rs_demo
