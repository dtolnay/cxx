#pragma once
#include "rust/cxx.h"
#include <memory>
#include <string>

namespace org {
namespace example {

class ThingC {
public:
  ThingC(std::string appname);
  ~ThingC();

  std::string appname;
};

struct SharedThing;

std::unique_ptr<ThingC> make_demo(rust::Str appname);
const std::string &get_name(const ThingC &thing);
void do_thing(SharedThing state);

} // namespace example
} // namespace org
