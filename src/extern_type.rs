/// A type for which the layout is determined by its C++ definition.
///
/// This trait serves the following two related purposes.
///
/// <br>
///
/// ## Safely unifying occurrences of the same extern type
///
/// `ExternType` makes it possible for CXX to safely share a consistent Rust
/// type across multiple #\[cxx::bridge\] invocations that refer to a common
/// extern C++ type.
///
/// In the following snippet, two #\[cxx::bridge\] invocations in different
/// files (possibly different crates) both contain function signatures involving
/// the same C++ type `example::Demo`. If both were written just containing
/// `type Demo;`, then both macro expansions would produce their own separate
/// Rust type called `Demo` and thus the compiler wouldn't allow us to take the
/// `Demo` returned by `file1::ffi::create_demo` and pass it as the `Demo`
/// argument accepted by `file2::ffi::take_ref_demo`. Instead, one of the two
/// `Demo`s has been defined as an extern type alias of the other, making them
/// the same type in Rust. The CXX code generator will use an automatically
/// generated `ExternType` impl emitted in file1 to statically verify that in
/// file2 `crate::file1::ffi::Demo` really does refer to the C++ type
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
/// ## Integrating with bindgen-generated types
///
/// Handwritten `ExternType` impls make it possible to plug in a data structure
/// emitted by bindgen as the definition of a C++ type emitted by CXX.
///
/// By writing the unsafe `ExternType` impl, the programmer asserts that the C++
/// namespace and type name given in the type id refers to a C++ type that is
/// equivalent to Rust type that is the `Self` type of the impl.
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
/// use cxx::{type_id, ExternType};
///
/// unsafe impl ExternType for folly_sys::StringPiece {
///     type Id = type_id!("folly::StringPiece");
///     type Kind = cxx::kind::Opaque;
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
///
/// <br><br>
///
/// ## Opaque and Trivial types
///
/// Some C++ types are safe to hold and pass around in Rust, by value.
/// Those C++ types must have a trivial move constructor, and must
/// have no destructor.
///
/// If you believe your C++ type is indeed trivial, you can specify
/// ```
/// # struct TypeName;
/// # unsafe impl cxx::ExternType for TypeName {
/// type Id = cxx::type_id!("name::space::of::TypeName");
/// type Kind = cxx::kind::Trivial;
/// # }
/// ```
/// which will enable you to pass it into C++ functions by value,
/// return it by value from such functions, and include it in
/// `struct`s that you have declared to `cxx::bridge`. Your promises
/// about the triviality of the C++ type will be checked using
/// `static_assert`s in the generated C++.
///
/// Opaque types can't be passed by value, but can still be held
/// in `UniquePtr`.
pub unsafe trait ExternType {
    /// A type-level representation of the type's C++ namespace and type name.
    ///
    /// This will always be defined using `type_id!` in the following form:
    ///
    /// ```
    /// # struct TypeName;
    /// # unsafe impl cxx::ExternType for TypeName {
    /// type Id = cxx::type_id!("name::space::of::TypeName");
    /// type Kind = cxx::kind::Opaque;
    /// # }
    /// ```
    type Id;

    /// Either `cxx::kind::Opaque` or `cxx::kind::Trivial`. If in doubt, use
    /// `cxx::kind::Opaque`.
    type Kind;
}

/// Marker types identifying Rust's knowledge about an extern C++ type.
///
/// These markers are used in the `Kind` associated type in impls of the
/// [`ExternType`] trait. Refer to the discussion of [Opaque and Trivial
/// types][trait] for an overview of their purpose.
///
/// [trait]: ExternType#opaque-and-trivial-types
pub mod kind {
    /// An opaque type which can't be passed or held by value within Rust.
    /// For example, a C++ type with a destructor, or a non-trivial move
    /// constructor. Rust's strict move semantics mean that we can't own
    /// these by value in Rust, but they can still be owned by a
    /// `UniquePtr`...
    pub struct Opaque;

    /// A type with trivial move constructors and no destructor, which
    /// can therefore be owned and moved around in Rust code directly.
    pub struct Trivial;
}

#[doc(hidden)]
pub fn verify_extern_type<T: ExternType<Id = Id>, Id>() {}

#[doc(hidden)]
pub fn verify_extern_kind<T: ExternType<Kind = Kind>, Kind>() {}
