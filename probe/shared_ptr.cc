#include <memory>

// https://stackoverflow.com/a/6830836/6086311 explains why this representation.
static_assert(sizeof(std::shared_ptr<void>) == 2 * sizeof(void *), "");
static_assert(alignof(std::shared_ptr<void>) == sizeof(void *), "");
