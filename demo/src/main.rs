#[cxx::bridge(namespace = "org::blobstore")]
mod ffi {
    // Shared structs with fields visible to both languages.
    struct BlobMetadata {
        size: usize,
        tags: Vec<String>,
    }

    /// A classic.
    enum BlobEnum {
        /// This is my doc
        Bar(i32),
        Baz(bool),
        Bam(BlobMetadata),
    }

    enum BlobCLike {
        Bar,
        Baz,
        Bam,
    }

    // Rust types and signatures exposed to C++.
    extern "Rust" {
        type MultiBuf;

        fn next_chunk(buf: &mut MultiBuf) -> &[u8];
    }

    // C++ types and signatures exposed to Rust.
    unsafe extern "C++" {
        include!("demo/include/blobstore.h");

        type BlobstoreClient;

        fn new_blobstore_client() -> UniquePtr<BlobstoreClient>;
        fn put(&self, parts: &mut MultiBuf) -> u64;
        fn tag(&self, blobid: u64, tag: &str);
        fn metadata(&self, blobid: u64) -> BlobMetadata;

        fn make_enum() -> BlobEnum;
        fn take_enum(enm: &BlobEnum);
        fn take_mut_enum(enm: &mut BlobEnum);
    }
}

// An iterator over contiguous chunks of a discontiguous file object.
//
// Toy implementation uses a Vec<Vec<u8>> but in reality this might be iterating
// over some more complex Rust data structure like a rope, or maybe loading
// chunks lazily from somewhere.
pub struct MultiBuf {
    chunks: Vec<Vec<u8>>,
    pos: usize,
}
pub fn next_chunk(buf: &mut MultiBuf) -> &[u8] {
    let next = buf.chunks.get(buf.pos);
    buf.pos += 1;
    next.map_or(&[], Vec::as_slice)
}

fn main() {
    let f = ffi::BlobEnum::Bar(1);
    ffi::take_enum(&f);
    let mut f = ffi::make_enum();
    match f {
        ffi::BlobEnum::Bar(val) => println!("The value is {val}"),
        ffi::BlobEnum::Baz(val) => println!("The value is {val}"),
        _ => {}
    }
    ffi::take_mut_enum(&mut f);
    match f {
        ffi::BlobEnum::Bar(val) => println!("The value is {val}"),
        ffi::BlobEnum::Baz(val) => println!("The value is {val}"),
        _ => {}
    }

    let client = ffi::new_blobstore_client();

    // Upload a blob.
    let chunks = vec![b"fearless".to_vec(), b"concurrency".to_vec()];
    let mut buf = MultiBuf { chunks, pos: 0 };
    let blobid = client.put(&mut buf);
    println!("blobid = {}", blobid);

    // Add a tag.
    client.tag(blobid, "rust");

    // Read back the tags.
    let metadata = client.metadata(blobid);
    println!("tags = {:?}", metadata.tags);
}
