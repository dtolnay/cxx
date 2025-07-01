#pragma once

// kj-rs/kj.h - Rust/C++ Interoperability Utilities for KJ
//
// This header provides seamless integration between Rust types and KJ (Cap'n Proto's C++ library)
// types, enabling efficient and safe data exchange across the language boundary.
//
// Key features:
// - Automatic string conversion via KJ_STRINGIFY functions
// - Automatic hash code generation via KJ_HASHCODE functions  
// - Zero-copy conversions between Rust and KJ container types
// - Utility structs for creating Rust types from KJ types
//
// Usage:
//   #include "kj-rs/kj.h"
//
// Example:
//   rust::String rustStr = getRustString();  // Call Rust function that returns rust::String
//   kj::String kjStr = kj::str(rustStr);      // Automatic conversion
//   auto hash = kj::hashCode(rustStr);        // Automatic hash computation
//
// ============================================================================
// Usage Examples and Best Practices
// ============================================================================
//
// Example 1: Zero-copy string processing
// ======================================
//
// // C++ side - processing a Rust string without copying
// void processRustString(const rust::String& rustStr) {
//   // rustStr comes from calling a Rust function that returns rust::String
//   // Automatic conversion to KJ string (uses rust::KJ_STRINGIFY via ADL)
//   kj::String kjStr = kj::str(rustStr);
//   
//   // Can also get raw access without null termination
//   kj::ArrayPtr<const char> chars = kj_rs::fromRust(rustStr);
//   
//   // Hash codes work automatically too
//   auto hash = kj::hashCode(rustStr);
// }
//
// Example 2: Container conversions
// ================================
//
// void processRustData(const rust::Vec<int>& rustVec) {
//   // rustVec comes from calling a Rust function that returns rust::Vec<int>
//   // Zero-copy view of Rust vector data
//   kj::ArrayPtr<const int> data = kj_rs::fromRust(rustVec);
//   
//   // Process with KJ algorithms
//   int sum = 0;
//   for (int x : data) sum += x;
// }
//
// Example 3: Providing KJ data to Rust
// =====================================
//
// void passDataToRust() {
//   kj::Array<double> kjArray = kj::heapArray<double>({1.1, 2.2, 3.3});
//   
//   // Read-only access for Rust
//   rust::Slice<const double> readOnlySlice = kj_rs::Rust::from(&kjArray);
//   
//   // Mutable access for Rust (allows modification of original array)
//   rust::Slice<double> mutableSlice = kj_rs::RustMutable::from(&kjArray);
//   
//   // Owned copy for Rust (independent of original array lifetime)
//   kj::ArrayPtr<const double> arrayPtr = kjArray.asPtr();
//   rust::Vec<double> ownedVec = kj_rs::RustCopy::from(&arrayPtr);
// }
//
// Example 4: String conversions with different ownership
// =======================================================
//
// void stringConversions() {
//   kj::String kjStr = kj::str("Hello, World!");
//   
//   // Zero-copy reference (fast, but tied to kjStr lifetime)
//   rust::String rustRef = kj_rs::Rust::from(&kjStr);
//   
//   // Independent copy (slower, but owned by Rust)
//   kj::StringPtr kjPtr = kjStr.asPtr();
//   rust::String rustOwned = kj_rs::RustCopy::from(&kjPtr);
//   
//   // Can safely destroy kjStr, rustOwned remains valid
// }
//
// Example 5: Complex type conversions
// ====================================
//
// void complexConversions(rust::Vec<rust::String> rustStrings) {
//   // rustStrings comes from calling a Rust function that returns rust::Vec<rust::String>
//   // Convert vector of Rust strings to KJ strings (with copying)
//   kj::Array<kj::String> kjStrings = kj_rs::fromRust(kj::mv(rustStrings));
//   // Each string is now properly null-terminated for KJ use
// }
//
// Performance Notes:
// ==================
// - fromRust() functions are zero-copy when possible
// - Rust::from() creates zero-copy views/references
// - RustCopy::from() always copies data for independent ownership
// - RustMutable::from() creates zero-copy mutable views
// - Automatic conversions via kj::str() and kj::hashCode() are efficient

#include <rust/cxx.h>

#include <kj/hash.h>
#include <kj/string.h>

namespace rust {

// ============================================================================
// Automatic KJ Integration Functions
// ============================================================================
//
// These functions enable automatic conversion of Rust string types when using
// KJ string and hash utilities. They use Argument Dependent Lookup (ADL) to
// be called automatically by kj::str() and kj::hashCode().

/// Converts rust::String to kj::ArrayPtr for use with KJ string functions.
/// 
/// This function is called automatically by kj::str() when passed a rust::String.
/// Since rust::String is not null-terminated, we return kj::ArrayPtr which
/// doesn't require null termination.
///
/// Example:
///   rust::String rustStr = getRustString();  // From Rust function call
///   kj::String kjStr = kj::str(rustStr);      // Calls this function automatically
inline auto KJ_STRINGIFY(const ::rust::String& str) {
  // HACK: rust::String is not null-terminated, so we use kj::ArrayPtr instead
  // which usually acts like kj::StringPtr but does not rely on null
  // termination.
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Converts rust::str (string slice) to kj::ArrayPtr for use with KJ string functions.
///
/// This function is called automatically by kj::str() when passed a rust::str.
/// Since rust::str is not null-terminated, we return kj::ArrayPtr which
/// doesn't require null termination.
///
/// Example:
///   rust::str rustStr = getRustStrSlice();  // From Rust function call
///   kj::String kjStr = kj::str(rustStr);     // Calls this function automatically
inline auto KJ_STRINGIFY(const ::rust::str& str) {
  // HACK: rust::str is not null-terminated, so we use kj::ArrayPtr instead
  // which usually acts like kj::StringPtr but does not rely on null
  // termination.
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Computes hash code for rust::String.
///
/// This function is called automatically by kj::hashCode() when passed a rust::String.
/// The hash is computed using KJ's standard string hashing algorithm.
///
/// Example:
///   rust::String rustStr = getRustString();  // From Rust function call
///   auto hash = kj::hashCode(rustStr);        // Calls this function automatically
inline auto KJ_HASHCODE(const ::rust::String& str) { 
  return kj::hashCode(kj::toCharSequence(str)); 
}

/// Computes hash code for rust::str (string slice).
///
/// This function is called automatically by kj::hashCode() when passed a rust::str.
/// The hash is computed using KJ's standard string hashing algorithm.
///
/// Example:
///   rust::str rustStr = getRustStrSlice();  // From Rust function call
///   auto hash = kj::hashCode(rustStr);       // Calls this function automatically
inline auto KJ_HASHCODE(const ::rust::str& str) { 
  return kj::hashCode(kj::toCharSequence(str)); 
}

}  // namespace rust

namespace kj_rs {

// ============================================================================
// Rust to KJ Conversion Functions
// ============================================================================
//
// These functions provide zero-copy conversions from Rust container types 
// to their KJ equivalents, enabling efficient data exchange across the
// language boundary.

/// Converts rust::Vec<T> to kj::ArrayPtr<const T> for zero-copy read access.
///
/// This provides a read-only view of the Rust vector's data without copying.
/// The ArrayPtr remains valid as long as the original Vec exists.
///
/// Example:
///   rust::Vec<int> rustVec = getRustVec();  // From Rust function call
///   kj::ArrayPtr<const int> ptr = fromRust(rustVec);
///   // Can now use ptr with KJ algorithms
template <typename T>
inline kj::ArrayPtr<const T> fromRust(const ::rust::Vec<T>& vec) {
  return kj::ArrayPtr<const T>(vec.data(), vec.size());
}

/// Converts rust::Slice<T> to kj::ArrayPtr<T> for zero-copy access.
///
/// This provides access to the Rust slice's data without copying.
/// The mutability depends on the original slice's mutability.
///
/// Example:
///   rust::Slice<int> rustSlice = getRustSlice();  // From Rust function call
///   kj::ArrayPtr<int> ptr = fromRust(rustSlice);
///   // Can now modify data through ptr
template <typename T>
inline kj::ArrayPtr<T> fromRust(const ::rust::Slice<T>& slice) {
  return kj::ArrayPtr<T>(slice.data(), slice.size());
}

/// Converts rust::String to kj::ArrayPtr<const char> for zero-copy string access.
///
/// This provides a read-only view of the Rust string's UTF-8 data without copying.
/// Note: The resulting ArrayPtr is NOT null-terminated.
///
/// Example:
///   rust::String rustStr = getRustString();  // From Rust function call
///   kj::ArrayPtr<const char> chars = fromRust(rustStr);
///   kj::String kjStr = kj::str(chars);        // Copy to null-terminated string
inline kj::ArrayPtr<const char> fromRust(const ::rust::String& str) {
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Converts rust::Str to kj::ArrayPtr<const char> for zero-copy string slice access.
///
/// This provides a read-only view of the Rust string slice's UTF-8 data without copying.
/// Note: The resulting ArrayPtr is NOT null-terminated.
///
/// Example:
///   rust::Str rustStr = getRustStrSlice();  // From Rust function call
///   kj::ArrayPtr<const char> chars = fromRust(rustStr);
///   kj::String kjStr = kj::str(chars);       // Copy to null-terminated string
inline kj::ArrayPtr<const char> fromRust(const ::rust::Str& str) {
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Converts rust::Vec<rust::String> to kj::Array<kj::String> with copying.
///
/// This function copies all strings from the Rust vector into a KJ array.
/// Each Rust string is converted to a proper null-terminated KJ string.
///
/// Example:
///   rust::Vec<rust::String> rustStrings = getRustStringVec();  // From Rust function call
///   kj::Array<kj::String> kjStrings = fromRust(kj::mv(rustStrings));
///   // kjStrings now contains copies of the strings
inline kj::Array<kj::String> fromRust(::rust::Vec<::rust::String> vec) {
  auto res = kj::heapArrayBuilder<kj::String>(vec.size());
  for (auto& entry: vec) {
    res.add(kj::str(entry.c_str()));
  }
  return res.finish();
}

// ============================================================================
// KJ to Rust Conversion Utility Structs
// ============================================================================
//
// These utility structs provide convenient ways to create Rust types from
// KJ types with different ownership and mutability semantics.

/// Creates Rust reference types (slices/views) from KJ types without copying.
///
/// Use this when you want to give Rust code a read-only view of KJ data
/// without transferring ownership or copying the data.
///
/// Example:
///   kj::Array<int> kjArray = kj::heapArray<int>({1, 2, 3});
///   rust::Slice<const int> rustSlice = Rust::from(&kjArray);
///   // Rust code can now read from kjArray through rustSlice
struct Rust {
  /// Creates a const slice from kj::ArrayPtr.
  template <typename T>
  static ::rust::Slice<const T> from(const kj::ArrayPtr<T>* arr) {
    return ::rust::Slice<const T>(arr->begin(), arr->size());
  }

  /// Creates a const slice from kj::Array.
  template <typename T>
  static ::rust::Slice<const T> from(const kj::Array<T>* arr) {
    return ::rust::Slice<const T>(arr->begin(), arr->size());
  }

  /// Creates a rust::String view from kj::String without copying.
  /// The resulting Rust string references the KJ string's memory.
  static ::rust::String from(const kj::String* str) {
    return ::rust::String(str->begin(), str->size());
  }

  /// Creates a rust::Str view from kj::StringPtr without copying.
  /// The resulting Rust string slice references the KJ string's memory.
  static ::rust::Str from(const kj::StringPtr* str) {
    return ::rust::Str(str->begin(), str->size());
  }
};

/// Creates owned Rust types from KJ types by copying the data.
///
/// Use this when you want Rust code to own independent copies of the data
/// that can outlive the original KJ objects.
///
/// Example:
///   kj::StringPtr kjStr = "Hello, World!";
///   rust::String rustStr = RustCopy::from(&kjStr);
///   // rustStr now owns its own copy of the string data
struct RustCopy {
  /// Creates an owned rust::String by copying from kj::StringPtr.
  static ::rust::String from(const kj::StringPtr* str) {
    return ::rust::String(str->begin(), str->size());
  }

  /// Creates an owned rust::Vec by copying from kj::ArrayPtr.
  template <typename T>
  static ::rust::Vec<T> from(kj::ArrayPtr<const T>* arr) {
    ::rust::Vec<T> result;
    result.reserve(arr->size());
    for (auto& t : *arr) {
      result.push_back(t);
    }
    return result;
  }
};

/// Creates mutable Rust slices from KJ types for modification.
///
/// Use this when you want Rust code to be able to modify the original
/// KJ data through mutable references.
///
/// Example:
///   kj::Array<int> kjArray = kj::heapArray<int>({1, 2, 3});
///   rust::Slice<int> rustSlice = RustMutable::from(&kjArray);
///   // Rust code can now modify kjArray through rustSlice
struct RustMutable {
  /// Creates a mutable slice from kj::ArrayPtr.
  template <typename T>
  static ::rust::Slice<T> from(kj::ArrayPtr<T>* arr) {
    return ::rust::Slice<T>(arr->begin(), arr->size());
  }

  /// Creates a mutable slice from kj::Array.
  template <typename T>
  static ::rust::Slice<T> from(kj::Array<T>* arr) {
    return ::rust::Slice<T>(arr->begin(), arr->size());
  }
};

}  // namespace kj_rs