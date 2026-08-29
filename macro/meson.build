pkg_macro = cargo.package('cxxbridge-macro')
macro = pkg_macro.proc_macro()
pkg_macro.override_dependency(declare_dependency(link_with: macro))
