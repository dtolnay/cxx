#pragma once
#include "rust/cxx.h"
#include <memory>

namespace org {
namespace blobstore {

struct MultiBuf;
struct BlobMetadata;
struct BlobEnum;

class BlobstoreClient {
public:
  BlobstoreClient();
  uint64_t put(MultiBuf &buf) const;
  void tag(uint64_t blobid, rust::Str tag) const;
  BlobMetadata metadata(uint64_t blobid) const;

private:
  class impl;
  std::shared_ptr<impl> impl;
};

std::unique_ptr<BlobstoreClient> new_blobstore_client();

BlobEnum make_enum();
void take_enum(const BlobEnum&);
void take_mut_enum(BlobEnum&);

} // namespace blobstore
} // namespace org
