#include "repro/src/lib.rs.h"
#include <memory>

void repro() {
  std::shared_ptr<RustType> client = std::make_shared<RustType>(*new_client());
  (void)client;
}
