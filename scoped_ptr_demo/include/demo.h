#pragma once
#include <boost/scoped_ptr.hpp>

struct Class {
  ~Class();
  void print() const;
  int32_t x;
};

using ScopedClass = boost::scoped_ptr<Class>;

void run();
