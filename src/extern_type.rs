/// A type for which the layout is determined by an external definition.
///
/// `ExternType` makes it possible for CXX to safely share a consistent Rust type across multiple
/// #\[cxx::bridge\] invocations, both for shared types defined in another bridge and external C++
/// definitions. This serves multiple related purposes.
///
/// <br>
///
/// ## Safely unifying occurrences of the same extern C++ type
///
/// In the following snippet, two #\[cxx::bridge\] invocations in different files (possibly
/// different crates) both contain function signatures involving the same C++ type `example::Demo`.
///
/// If both were written just containing `type Demo;`, then both macro expansions would produce
/// their own separate Rust type called `Demo` and thus the compiler wouldn't allow us to take the
/// `Demo` returned by `file1::ffi::create_demo` and pass it as the `Demo` argument accepted by
/// `file2::ffi::take_ref_demo`. Instead, one of the two `Demo`s has been defined as an extern type
/// alias of the other, making them the same type in Rust.
///
/// The CXX code generator will use an automatically generated `ExternType` impl emitted in file1 to
/// statically verify that in file2 `crate::file1::ffi::Demo` really does refer to the C++ type
/// `example::Demo` as expected in file2.
///
/// ```no_run
/// // file1.rs
/// # mod file1 {
/// #[cxx::bridge(namespace = example)]
/// pub mod ffi {
///     extern "C" {
///         type Demo;
///
///         fn create_demo() -> UniquePtr<Demo>;
///     }
/// }
/// # }
///
/// // file2.rs
/// #[cxx::bridge(namespace = example)]
/// pub mod ffi {
///     extern "C" {
///         type Demo = crate::file1::ffi::Demo;
///
///         fn take_ref_demo(demo: &Demo);
///     }
/// }
/// #
/// # fn main() {}
/// ```
///
/// <br><br>
///
/// ## Reusing Rust/C++ shared types across multiple bridges
///
/// `ExternType` enables reusing a shared Rust/C++ type declared in another bridge module, allowing
/// for the creation of libraries to wrap types used in multiple different bridges.
///
/// Imagine we have an existing move-only C++ type, file::UniqueFd, that wraps sole ownership of a
/// file descriptor, analogous to Rust's std::fd::File. The example below defines a shared type
/// `File` that allows safely transferring ownership of the file across the interface without Box or
/// UniquePtr and without resource leaks. This type can then be reused in other bridges.
///
/// ```no_run
/// // file/src/lib.rs
/// # #[cfg(unix)]
/// # mod file {
/// # use std::os::unix::io::{IntoRawFd, FromRawFd};
/// #[cxx::bridge(namespace = file::ffi)]
/// pub mod ffi {
///     /// A file backed by a file descriptor, which it is the sole owner of.
///     struct File {
///         fd: i32,
///     }
/// }
///
/// impl From<ffi::File> for std::fs::File {
///     fn from(value: ffi::File) -> Self {
///         // Safe because ffi::File owns its file descriptor.
///         unsafe { Self::from_raw_fd(value.fd) }
///     }
/// }
///
/// impl From<std::fs::File> for ffi::File {
///     fn from(value: std::fs::File) -> Self {
///         Self { fd: value.into_raw_fd() }
///     }
/// }
///
/// impl Drop for ffi::File {
///     fn drop(&mut self) {
///         // Safe because ffi::File owns its file descriptor.
///         unsafe { std::fs::File::from_raw_fd(self.fd); }
///     }
/// }
/// # }
///
/// // file/src/lib.h
/// # /*
/// namespace file {
///
/// ffi::File TransferToFFI(File file) {
///     // Imagine file::UniqueFd::release() is analogous to from_raw_fd
///     return ffi::File{ .fd = file.release() };
/// }
///
/// }
/// # */
///
/// // TODO(https://github.com/dtolnay/cxx/pull/298): Currently this bridge must use the same
/// // namespace as any bridge it creates aliases from.
///
/// // usage.rs
/// # #[cfg(unix)]
/// # mod usage {
/// #[cxx::bridge(namespace = file::ffi)]
/// pub mod ffi {
///     type File = crate::file::ffi::File;
///
///     extern "C" {
///         type Demo;
///
///         fn create_demo(file: File) -> UniquePtr<Demo>;
///     }
/// }
/// # }
///
/// // usage.cc
/// # /*
/// file::ffi::File ConvertFile(file::UniqueFd file) {
/// }
///
/// void CreateDemo(file::UniqueFd file) {
///     auto demo = ffi::create_demo(file::TransferToFFI(std::move(file)));
///     // use demo
/// }
/// # */
///
/// # fn main() {}
/// ```
///
/// <br><br>
///
/// ## Integrating with bindgen-generated types
///
/// Handwritten `ExternType` impls make it possible to plug in a data structure emitted by bindgen
/// as the definition of an opaque C++ type emitted by CXX.
///
/// By writing the unsafe `ExternType` impl, the programmer asserts that the C++ namespace and type
/// name given in the type id refers to a C++ type that is equivalent to Rust type that is the
/// `Self` type of the impl.
///
/// ```no_run
/// # const _: &str = stringify! {
/// mod folly_sys;  // the bindgen-generated bindings
/// # };
/// # mod folly_sys {
/// #     #[repr(transparent)]
/// #     pub struct StringPiece([usize; 2]);
/// # }
///
/// unsafe impl cxx::ExternType for folly_sys::StringPiece {
///     type Kind = cxx::ExternTypeKindOpaqueCpp;
///     type Id = cxx::type_id!("folly::StringPiece");
/// }
///
/// #[cxx::bridge(namespace = folly)]
/// pub mod ffi {
///     extern "C" {
///         include!("rust_cxx_bindings.h");
///
///         type StringPiece = crate::folly_sys::StringPiece;
///
///         fn print_string_piece(s: &StringPiece);
///     }
/// }
///
/// // Now if we construct a StringPiece or obtain one through one
/// // of the bindgen-generated signatures, we are able to pass it
/// // along to ffi::print_string_piece.
/// #
/// # fn main() {}
/// ```
pub unsafe trait ExternType {
    /// The type's kind.
    ///
    /// Must be either:
    ///   * `ExternTypeKindShared` for a shared type declared outside of an extern block in a
    ///     cxx::bridge, or
    ///   * `ExternTypeKindOpqaueCpp` for an opaque C++ type declared inside of an `extern "C"`
    ///     block.
    ///
    /// Opaque Rust type aliases are unsupported because they can included with a use declaration
    /// and aliased more simply outside of the cxx::bridge.
    type Kind;

    /// A type-level representation of the type's C++ namespace and type name.
    ///
    /// This will always be defined using `type_id!` in the following form:
    ///
    /// ```
    /// # struct TypeName;
    /// # unsafe impl cxx::ExternType for TypeName {
    /// # type Kind = cxx::ExternTypeKindOpaqueCpp;
    /// type Id = cxx::type_id!("name::space::of::TypeName");
    /// # }
    /// ```
    type Id;
}

pub struct ExternTypeKindOpaqueCpp;
pub struct ExternTypeKindShared;

#[doc(hidden)]
pub fn verify_extern_type<T: ExternType<Kind = Kind, Id = Id>, Kind, Id>() {}
