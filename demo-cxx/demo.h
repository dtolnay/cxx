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
struct JsonBlob;

std::unique_ptr<ThingC> make_demo(::rust::Str appname);
const std::string &get_name(const ThingC &thing);
std::unique_ptr<std::vector<uint8_t>> do_thing(SharedThing state);
JsonBlob get_jb(const ::rust::Vec<uint8_t>& vec);

} // namespace example
} // namespace org
