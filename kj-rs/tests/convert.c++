#include "kj-rs/convert.h"

#include <rust/cxx.h>

#include <kj/array.h>
#include <kj/string.h>
#include <kj/test.h>

// Test kj-rs integration utilities for Rust/C++ interoperability

namespace kj_rs {

KJ_TEST("rust::String with kj::str") {
  // Create a rust::String
  ::rust::String rustStr("Hello, World!");

  // Test that kj::str() automatically uses our KJ_STRINGIFY function
  auto kjStr = kj::str(rustStr);

  KJ_EXPECT(kjStr == "Hello, World!");
  KJ_EXPECT(kjStr.size() == rustStr.size());
}

KJ_TEST("rust::str with kj::str") {
  // Create a rust::str
  ::rust::str rustStr("Rust string slice");

  // Test that kj::str() automatically uses our KJ_STRINGIFY function
  auto kjStr = kj::str(rustStr);

  KJ_EXPECT(kjStr == "Rust string slice");
  KJ_EXPECT(kjStr.size() == rustStr.size());
}

KJ_TEST("rust::String with kj::hashCode") {
  ::rust::String rustStr("hash test");
  kj::StringPtr kjStr = "hash test";

  // Test that kj::hashCode() automatically uses our KJ_HASHCODE function
  auto rustHash = kj::hashCode(rustStr);
  auto kjHash = kj::hashCode(kjStr);

  KJ_EXPECT(rustHash == kjHash);
}

KJ_TEST("rust::str with kj::hashCode") {
  ::rust::str rustStr("hash test slice");
  kj::StringPtr kjStr = "hash test slice";

  // Test that kj::hashCode() automatically uses our KJ_HASHCODE function
  auto rustHash = kj::hashCode(rustStr);
  auto kjHash = kj::hashCode(kjStr);

  KJ_EXPECT(rustHash == kjHash);
}

KJ_TEST("from<Rust> Vec conversion") {
  // Create a rust::Vec with some data
  ::rust::Vec<int> rustVec;
  rustVec.push_back(1);
  rustVec.push_back(2);
  rustVec.push_back(3);

  // Convert to kj::ArrayPtr
  auto arrayPtr = from<Rust>(rustVec);

  KJ_EXPECT(arrayPtr.size() == 3);
  KJ_EXPECT(arrayPtr[0] == 1);
  KJ_EXPECT(arrayPtr[1] == 2);
  KJ_EXPECT(arrayPtr[2] == 3);
}

KJ_TEST("from<Rust> Slice conversion") {
  // Create some data and a rust::Slice
  int data[] = {10, 20, 30, 40};
  ::rust::Slice<int> rustSlice(data, 4);

  // Convert to kj::ArrayPtr
  auto arrayPtr = from<Rust>(rustSlice);

  KJ_EXPECT(arrayPtr.size() == 4);
  KJ_EXPECT(arrayPtr[0] == 10);
  KJ_EXPECT(arrayPtr[1] == 20);
  KJ_EXPECT(arrayPtr[2] == 30);
  KJ_EXPECT(arrayPtr[3] == 40);
}

KJ_TEST("from<Rust> String conversion") {
  ::rust::String rustStr("Convert me!");

  auto arrayPtr = from<Rust>(rustStr);
  auto kjStr = kj::str(arrayPtr);

  KJ_EXPECT(kjStr == "Convert me!");
  KJ_EXPECT(arrayPtr.size() == rustStr.size());
}

KJ_TEST("from<Rust> Str conversion") {
  ::rust::Str rustStr("String slice conversion");

  auto arrayPtr = from<Rust>(rustStr);
  auto kjStr = kj::str(arrayPtr);

  KJ_EXPECT(kjStr == "String slice conversion");
  KJ_EXPECT(arrayPtr.size() == rustStr.size());
}

KJ_TEST("from<RustCopy> Slice<str> conversion") {
  ::rust::str strings[] = {"first", "second", "third"};
  ::rust::Slice<::rust::str> rustSlice(strings, 3);

  auto kjArray = from<RustCopy>(rustSlice);

  KJ_EXPECT(kjArray.size() == 3);
  KJ_EXPECT(kjArray[0] == "first");
  KJ_EXPECT(kjArray[1] == "second");
  KJ_EXPECT(kjArray[2] == "third");
}

KJ_TEST("from<RustCopy> Vec<String> conversion") {
  ::rust::Vec<::rust::String> rustVec;
  rustVec.emplace_back("first");
  rustVec.emplace_back("second");
  rustVec.emplace_back("third");

  auto kjArray = from<RustCopy>(rustVec);

  KJ_EXPECT(kjArray.size() == 3);
  KJ_EXPECT(kjArray[0] == "first");
  KJ_EXPECT(kjArray[1] == "second");
  KJ_EXPECT(kjArray[2] == "third");
}

KJ_TEST("Rust string - ArrayPtr to Slice conversion") {
  kj::Array<int> kjArray = kj::heapArray<int>({1, 2, 3, 4, 5});
  kj::ArrayPtr<int> arrayPtr = kjArray.asPtr();

  auto rustSlice = arrayPtr.as<Rust>();

  KJ_EXPECT(rustSlice.size() == 5);
  KJ_EXPECT(rustSlice[0] == 1);
  KJ_EXPECT(rustSlice[4] == 5);
}

KJ_TEST("Rust string - Array to Slice conversion") {
  kj::Array<int> kjArray = kj::heapArray<int>({10, 20, 30});

  auto rustSlice = kjArray.as<Rust>();

  KJ_EXPECT(rustSlice.size() == 3);
  KJ_EXPECT(rustSlice[0] == 10);
  KJ_EXPECT(rustSlice[1] == 20);
  KJ_EXPECT(rustSlice[2] == 30);
}

KJ_TEST("Rust string - String conversion") {
  kj::String kjStr = kj::str("KJ to Rust string");

  auto rustString = kjStr.as<Rust>();

  KJ_EXPECT(rustString.size() == kjStr.size());
  // Compare content by converting back
  auto convertedBack = kj::str(rustString);
  KJ_EXPECT(convertedBack == kjStr);
}

KJ_TEST("Rust string - StringPtr conversion") {
  kj::StringPtr kjStrPtr = "StringPtr to Rust";

  auto rustStr = kjStrPtr.as<Rust>();

  KJ_EXPECT(rustStr.size() == kjStrPtr.size());
  auto convertedBack = kj::str(rustStr);
  KJ_EXPECT(convertedBack == kjStrPtr);
}

KJ_TEST("RustCopy struct - StringPtr conversion") {
  kj::StringPtr kjStrPtr = "Copy this string";

  auto rustString = kjStrPtr.as<RustCopy>();

  KJ_EXPECT(rustString.size() == kjStrPtr.size());
  auto convertedBack = kj::str(rustString);
  KJ_EXPECT(convertedBack == kjStrPtr);
}

KJ_TEST("RustCopy struct - ArrayPtr conversion") {
  kj::Array<double> kjArray = kj::heapArray<double>({1.1, 2.2, 3.3});
  kj::ArrayPtr<const double> arrayPtr = kjArray.asPtr();

  auto rustVec = arrayPtr.as<RustCopy>();

  KJ_EXPECT(rustVec.size() == 3);
  KJ_EXPECT(rustVec[0] == 1.1);
  KJ_EXPECT(rustVec[1] == 2.2);
  KJ_EXPECT(rustVec[2] == 3.3);
}

KJ_TEST("RustMutable struct - ArrayPtr conversion") {
  kj::Array<int> kjArray = kj::heapArray<int>({100, 200, 300});
  kj::ArrayPtr<int> arrayPtr = kjArray.asPtr();

  auto rustSlice = arrayPtr.as<RustMutable>();

  KJ_EXPECT(rustSlice.size() == 3);
  KJ_EXPECT(rustSlice[0] == 100);
  KJ_EXPECT(rustSlice[1] == 200);
  KJ_EXPECT(rustSlice[2] == 300);

  // Test that we can modify through the mutable slice
  rustSlice[0] = 999;
  KJ_EXPECT(kjArray[0] == 999);  // Should be modified in original array
}

KJ_TEST("RustMutable struct - Array conversion") {
  kj::Array<char> kjArray = kj::heapArray<char>({'a', 'b', 'c'});

  auto rustSlice = kjArray.as<RustMutable>();

  KJ_EXPECT(rustSlice.size() == 3);
  KJ_EXPECT(rustSlice[0] == 'a');
  KJ_EXPECT(rustSlice[1] == 'b');
  KJ_EXPECT(rustSlice[2] == 'c');

  // Test mutability
  rustSlice[1] = 'X';
  KJ_EXPECT(kjArray[1] == 'X');
}

KJ_TEST("kj::ConstString as<Rust> conversion") {
  kj::ConstString kjConstStr = "ConstString test"_kjc;

  auto rustStr = kjConstStr.as<Rust>();

  KJ_EXPECT(rustStr.size() == kjConstStr.size());
  auto convertedBack = kj::str(rustStr);
  KJ_EXPECT(convertedBack == kjConstStr);
}

KJ_TEST("kj::ConstString as<RustCopy> conversion") {
  kj::ConstString kjConstStr = "Copy ConstString test"_kjc;

  auto rustString = kjConstStr.as<RustCopy>();

  KJ_EXPECT(rustString.size() == kjConstStr.size());
  auto convertedBack = kj::str(rustString);
  KJ_EXPECT(convertedBack == kjConstStr);
}

KJ_TEST("kj::LiteralStringConst as<Rust> conversion") {
  auto kjLiteralStr = "LiteralStringConst test"_kjc;

  auto rustStr = kjLiteralStr.as<Rust>();

  KJ_EXPECT(rustStr.size() == kjLiteralStr.size());
  auto convertedBack = kj::str(rustStr);
  KJ_EXPECT(convertedBack == kjLiteralStr);
}

KJ_TEST("kj::LiteralStringConst as<RustCopy> conversion") {
  auto kjLiteralStr = "Copy LiteralStringConst test"_kjc;

  auto rustString = kjLiteralStr.as<RustCopy>();

  KJ_EXPECT(rustString.size() == kjLiteralStr.size());
  auto convertedBack = kj::str(rustString);
  KJ_EXPECT(convertedBack == kjLiteralStr);
}

}  // namespace kj_rs
