#pragma once
#include <string>

namespace org {
namespace example {

class ThingC {
public:
  ThingC(std::string appname);
  ~ThingC();

  std::string appname;
};


void print_name(const ThingC &thing);

} // namespace example
} // namespace org
