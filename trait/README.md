This directory contains public CXX traits like ExternType and all related types needed to implement said traits. Most users will consume these types through the larger cxx crate, which re-exports them.

Users who need it can depend on this crate directly to implement ExternType for types in their
own crates without depending on the larger cxx crate. This avoids the need to link in cxx's C++
symbols where they are not needed. For example, a widely used library crate can depend on this,
enabling but not requiring its users to use the library's types with cxx.