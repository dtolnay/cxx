#include "demo-dyntrait/src/main.rs"

BoxDynMyData::BoxDynMyData(BoxDynMyData &&other) noexcept : repr(other.repr) {
  other.repr = {0, 0};
}

BoxDynMyData::~BoxDynMyData() noexcept {
  if (repr != std::array<std::uintptr_t, 2>{0, 0}) {
    dyn_mydata_drop_in_place(this);
  }
}

void BoxDynMyData::traitfn() const noexcept {
  dyn_mydata_traitfn(*this);
}

int main() {
  auto mydata = read_data();
  mydata.traitfn();
}
