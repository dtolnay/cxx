#pragma once

// kj-rs/convert.h - Rust/C++ Interoperability Utilities for KJ
//
// This header provides seamless integration between Rust types and KJ (Cap'n Proto's C++ library)
// types, enabling efficient and safe data exchange across the language boundary.
//
// ============================================================================
// USAGE OVERVIEW
// ============================================================================
//
// Converting C++ kj arrays, strings, etc to Rust:
// - kjObject.as<Rust>() - creates zero-copy read-only Rust view
// - kjObject.as<RustMutable>() - creates zero-copy mutable Rust view
// - kjObject.as<RustCopy>() - creates owned Rust copy
//
// Converting Rust to C++ kj objects:
// - from<Rust>(rustObject) - creates zero-copy C++ view
// - from<RustCopy>(rustObject) - creates owned C++ copy
// - kj::str(rustString) - automatic conversion (via KJ_STRINGIFY)
// - kj::hashCode(rustString) - automatic hash computation (via KJ_HASHCODE)
//
// ============================================================================
// CONVERSION FUNCTIONS
// ============================================================================
//
// Zero-copy conversions from Rust to C++:
// - from<Rust>(rust::Vec<T>) -> kj::ArrayPtr<const T>
// - from<Rust>(rust::Slice<T>) -> kj::ArrayPtr<T>
// - from<Rust>(rust::String) -> kj::ArrayPtr<const char>
// - from<Rust>(rust::str) -> kj::ArrayPtr<const char>
//
// Owned conversions from Rust to C++:
// - from<RustCopy>(rust::Slice<rust::str>) -> kj::Array<kj::String>
// - from<RustCopy>(rust::Vec<rust::String>) -> kj::Array<kj::String>
//
// Zero-copy conversions from C++ to Rust (read-only):
// - kjArray.as<Rust>() -> rust::Slice<const T>
// - kjString.as<Rust>() -> rust::String
// - kjStringPtr.as<Rust>() -> rust::str
// - kjConstString.as<Rust>() -> rust::str
//
// Zero-copy conversions from C++ to Rust (mutable):
// - kjArray.as<RustMutable>() -> rust::Slice<T>
// - kjArrayPtr.as<RustMutable>() -> rust::Slice<T>
//
// Owned conversions from C++ to Rust (copying):
// - kjStringPtr.as<RustCopy>() -> rust::String
// - kjConstString.as<RustCopy>() -> rust::String
// - kjArrayPtr.as<RustCopy>() -> rust::Vec<T>
//
// Automatic conversions (via ADL):
// - kj::str(rust::String) - uses KJ_STRINGIFY for seamless string conversion
// - kj::hashCode(rust::String) - uses KJ_HASHCODE for hash computation
//
// ============================================================================
// EXAMPLES
// ============================================================================
//
// Basic usage patterns:
//
// // Convert Rust to C++:
// kj::ArrayPtr<const int> cppView = from<Rust>(rustVec);
//
// // Convert C++ to Rust (read-only):
// rust::Slice<const int> rustView = cppArray.as<Rust>();
//
// // Convert C++ to Rust (mutable):
// rust::Slice<int> rustMutableView = cppArray.as<RustMutable>();
//
// // Convert C++ to Rust (copying):
// rust::String rustOwnedStr = cppStr.as<RustCopy>();
//
// // Automatic string conversion:
// kj::String cppStr = kj::str(rustStr);  // via KJ_STRINGIFY
//

#include <rust/cxx.h>

#include <kj/common.h>
#include <kj/hash.h>
#include <kj/string.h>

namespace rust {

// Automatic KJ Integration Functions - enable ADL for kj::str() and kj::hashCode()

/// Converts rust::String to kj::ArrayPtr - called by kj::str(rustString)
inline auto KJ_STRINGIFY(const ::rust::String& str) {
  // HACK: rust::String is not null-terminated, so we use kj::ArrayPtr instead
  // which usually acts like kj::StringPtr but does not rely on null
  // termination.
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Converts rust::str to kj::ArrayPtr - called by kj::str(rustStr)
inline auto KJ_STRINGIFY(const ::rust::str& str) {
  // HACK: rust::str is not null-terminated, so we use kj::ArrayPtr instead
  // which usually acts like kj::StringPtr but does not rely on null
  // termination.
  return kj::ArrayPtr<const char>(str.data(), str.size());
}

/// Hash code for rust::String - called by kj::hashCode(rustString)
inline auto KJ_HASHCODE(const ::rust::String& str) {
  return kj::hashCode(kj::toCharSequence(str));
}

/// Hash code for rust::str - called by kj::hashCode(rustStr)
inline auto KJ_HASHCODE(const ::rust::str& str) {
  return kj::hashCode(kj::toCharSequence(str));
}

}  // namespace rust

namespace kj_rs {

// KJ to Rust conversion utilities with different ownership semantics

/// Template function for nicer syntax: from<Rust>(rustObject) instead of fromRust(rustObject)
template <typename T, typename U>
inline auto from(U&& rustObject) {
  return T::into(std::forward<U>(rustObject));
}

/// Zero-copy read-only Rust views: kjObject.as<Rust>() and from<Rust>(kjObject)
struct Rust {
  /// kjArrayPtr.as<Rust>() - via Rust::from(&kjArrayPtr)
  template <typename T>
  static ::rust::Slice<const T> from(const kj::ArrayPtr<T>* arr) {
    return ::rust::Slice<const T>(arr->begin(), arr->size());
  }

  /// kjArray.as<Rust>() - via Rust::from(&kjArray)
  template <typename T>
  static ::rust::Slice<const T> from(const kj::Array<T>* arr) {
    return ::rust::Slice<const T>(arr->begin(), arr->size());
  }

  /// kjString.as<Rust>() - via Rust::from(&kjString)
  static ::rust::String from(const kj::String* str) {
    return ::rust::String(str->begin(), str->size());
  }

  /// kjStringPtr.as<Rust>() - via Rust::from(&kjStringPtr)
  static ::rust::Str from(const kj::StringPtr* str) {
    return ::rust::Str(str->begin(), str->size());
  }

  /// kjConstString.as<Rust>() - via Rust::from(&kjConstString)
  static ::rust::Str from(const kj::ConstString* str) {
    return ::rust::Str(str->begin(), str->size());
  }

  // into() methods for from<Rust>(rustObject) - converting Rust to KJ

  /// from<Rust>(rustVec) - Zero-copy read-only view
  template <typename T>
  static kj::ArrayPtr<const T> into(const ::rust::Vec<T>& vec) {
    return kj::ArrayPtr<const T>(vec.data(), vec.size());
  }

  /// from<Rust>(rustSlice) - Zero-copy slice view
  template <typename T>
  static kj::ArrayPtr<T> into(const ::rust::Slice<T>& slice) {
    return kj::ArrayPtr<T>(slice.data(), slice.size());
  }

  /// from<Rust>(rustString) - Zero-copy string chars (not null-terminated)
  static kj::ArrayPtr<const char> into(const ::rust::String& str) {
    return kj::ArrayPtr<const char>(str.data(), str.size());
  }

  /// from<Rust>(rustStr) - Zero-copy string slice chars (not null-terminated)
  static kj::ArrayPtr<const char> into(const ::rust::Str& str) {
    return kj::ArrayPtr<const char>(str.data(), str.size());
  }
};

/// Owned Rust copies: kjObject.as<RustCopy>() and from<RustCopy>(kjObject)
struct RustCopy {
  /// kjStringPtr.as<RustCopy>() - via RustCopy::from(&kjStringPtr)
  static ::rust::String from(const kj::StringPtr* str) {
    return ::rust::String(str->begin(), str->size());
  }

  /// kjConstString.as<RustCopy>() - via RustCopy::from(&kjConstString)
  static ::rust::String from(const kj::ConstString* str) {
    return ::rust::String(str->begin(), str->size());
  }

  /// kjArrayPtr.as<RustCopy>() - via RustCopy::from(&kjArrayPtr)
  template <typename T>
  static ::rust::Vec<T> from(kj::ArrayPtr<const T>* arr) {
    ::rust::Vec<T> result;
    result.reserve(arr->size());
    for (auto& t: *arr) {
      result.push_back(t);
    }
    return result;
  }

  /// from<RustCopy>(rustSliceOfStrs) - Copy slice of strs to null-terminated KJ strings
  static kj::Array<kj::String> into(::rust::Slice<::rust::str> slice) {
    auto res = kj::heapArrayBuilder<kj::String>(slice.size());
    for (auto& entry: slice) {
      res.add(kj::str(entry));
    }
    return res.finish();
  }

  /// from<RustCopy>(rustVecOfStrings) - Copy string vector to null-terminated KJ strings
  static kj::Array<kj::String> into(const ::rust::Vec<::rust::String>& vec) {
    auto res = kj::heapArrayBuilder<kj::String>(vec.size());
    for (auto& entry: vec) {
      res.add(kj::str(entry));
    }
    return res.finish();
  }
};

/// Mutable Rust views: kjObject.as<RustMutable>() and from<RustMutable>(kjObject)
struct RustMutable {
  /// kjArrayPtr.as<RustMutable>() - via RustMutable::from(&kjArrayPtr)
  template <typename T>
  static ::rust::Slice<T> from(kj::ArrayPtr<T>* arr) {
    return ::rust::Slice<T>(arr->begin(), arr->size());
  }

  /// kjArray.as<RustMutable>() - via RustMutable::from(&kjArray)
  template <typename T>
  static ::rust::Slice<T> from(kj::Array<T>* arr) {
    return ::rust::Slice<T>(arr->begin(), arr->size());
  }
};

}  // namespace kj_rs