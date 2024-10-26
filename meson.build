project(
  'cxx',
  default_options : ['rust_std=2021'],
  license : 'MIT OR Apache-2.0',
  license_files : ['LICENSE-APACHE', 'LICENSE-MIT'],
  meson_version : '>= 1.3.0',
)

add_languages('rust', native : true)
add_languages('rust', 'cpp', native : false)

rust = import('rust')

unicode_ident = subproject('unicode-ident-1-rs')
proc_macro2 = subproject('proc-macro2-1-rs')
quote = subproject('quote-1-rs')
syn = subproject('syn-2-rs')

rust.proc_macro(
  'cxxbridge_macro',
  sources : 'macro/src/lib.rs',
  dependencies : [
    proc_macro2.get_variable('dep'),
    quote.get_variable('dep'),
    syn.get_variable('dep'),
  ],
)
