{{#title Built-in bindings — Rust ♡ C++}}
# Built-in bindings reference

In addition to all the primitive types (`i32` &harr; `int32_t`), the following
common types may be used in the fields of shared structs and the arguments and
returns of extern functions.

<br>

<table>
<tr><th>name in Rust</th><th>name in C++</th><th>restrictions</th></tr>
<tr><td><code>String</code></td><td><code>rust::String</code></td><td></td></tr>
<tr><td><code>&amp;str</code></td><td><code>rust::Str</code></td><td></td></tr>
<tr><td><code>&amp;[T]</code></td><td><code>rust::Slice&lt;const T&gt;</code></td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><code>&amp;mut [T]</code></td><td><code>rust::Slice&lt;T&gt;</code></td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><a href="https://docs.rs/cxx/1.0/cxx/struct.CxxString.html"><code>CxxString</code></a></td><td><code>std::string</code></td><td><sup><i>cannot be passed by value</i></sup></td></tr>
<tr><td><code>Box&lt;T&gt;</code></td><td><code>rust::Box&lt;T&gt;</code></td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><a href="https://docs.rs/cxx/1.0/cxx/struct.UniquePtr.html"><code>UniquePtr&lt;T&gt;</code></a></td><td><code>std::unique_ptr&lt;T&gt;</code></td><td><sup><i>cannot hold opaque Rust type</i></sup></td></tr>
<tr><td><a href="https://docs.rs/cxx/1.0/cxx/struct.SharedPtr.html"><code>SharedPtr&lt;T&gt;</code></a></td><td><code>std::shared_ptr&lt;T&gt;</code></td><td><sup><i>cannot hold opaque Rust type</i></sup></td></tr>
<tr><td><code>[T; N]</code></td><td><code>std::array&lt;T, N&gt;</code></td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><code>Vec&lt;T&gt;</code></td><td><code>rust::Vec&lt;T&gt;</code></td><td><sup><i>cannot hold opaque C++ type</i></sup></td></tr>
<tr><td><a href="https://docs.rs/cxx/1.0/cxx/struct.CxxVector.html"><code>CxxVector&lt;T&gt;</code></a></td><td><code>std::vector&lt;T&gt;</code></td><td><sup><i>cannot be passed by value, cannot hold opaque Rust type</i></sup></td></tr>
<tr><td><code>*mut T</code>, <code>*const T</code></td><td><code>T*</code>, <code>const T*</code></td><td><sup><i>fn with a raw pointer argument must be declared unsafe to call</i></sup></td></tr>
<tr><td><code>fn(T, U) -&gt; V</code></td><td><code>rust::Fn&lt;V(T, U)&gt;</code></td><td><sup><i>only passing from Rust to C++ is implemented so far</i></sup></td></tr>
<tr><td><code>Result&lt;T&gt;</code></td><td><code>throw</code>/<code>catch</code></td><td><sup><i>allowed as return type only</i></sup></td></tr>
</table>

<br>

The C++ API of the `rust` namespace is defined by the *`include/cxx.h`* file in
the CXX GitHub repo. You will need to include this header in your C++ code when
working with those types. **When using Cargo and the cxx-build crate, the header
is made available to you at `#include "rust/cxx.h"`.**

The `rust` namespace additionally provides lowercase type aliases of all the
types mentioned in the table, for use in codebases preferring that style. For
example `rust::String`, `rust::Vec` may alternatively be written `rust::string`,
`rust::vec` etc.

## Pending bindings

The following types are intended to be supported "soon" but are just not
implemented yet. I don't expect any of these to be hard to make work but it's a
matter of designing a nice API for each in its non-native language.

<br>

<table>
<tr><th>name in Rust</th><th>name in C++</th></tr>
<tr><td><code>std::collections::BTreeMap&lt;K, V&gt;</code></td><td><sup><i>tbd</i></sup></td></tr>
<tr><td><code>std::collections::HashMap&lt;K, V&gt;</code></td><td><sup><i>tbd</i></sup></td></tr>
<tr><td><code>std::sync::Arc&lt;T&gt;</code></td><td><sup><i>tbd</i></sup></td></tr>
<tr><td><code>Option&lt;T&gt;</code></td><td><sup><i>tbd</i></sup></td></tr>
<tr><td><sup><i>tbd</i></sup></td><td><code>std::map&lt;K, V&gt;</code></td></tr>
<tr><td><sup><i>tbd</i></sup></td><td><code>std::unordered_map&lt;K, V&gt;</code></td></tr>
</table>
