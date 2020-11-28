#pragma once
#include <boost/scoped_ptr.hpp>
#include <type_traits>

template <typename T>
class rust_scoped_ptr {
public:
  using IsRelocatable = std::true_type;
  void drop_in_place() { this->~rust_scoped_ptr(); }
  boost::scoped_ptr<T> ptr;
  void *padding[2] = {};
};

struct Class {
  ~Class();
  void print() const;
  int32_t x;
};

using ScopedClass = rust_scoped_ptr<Class>;

rust_scoped_ptr<Class> getclass() noexcept;
