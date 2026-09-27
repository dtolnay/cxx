project('cxx', ['cpp', 'rust'], version: '1.0.194',
        meson_version: '>=1.12.0',
        default_options: ['cpp_std=gnu++20,c++20'])

# Ask user to generate Cargo.lock
fs = import('fs')
if not fs.exists(meson.project_source_root() / 'Cargo.lock')
  error('Cargo.lock missing, please run "cargo metadata > /dev/null" to generate it.')
endif

# Globals
incdir = include_directories('.')
rustc = meson.get_compiler('rust')
rust = import('rust')
cargo = rust.workspace(extra_members: ['gen/cmd', 'demo'])
pkg_cxx = cargo.package()

# Environment checks
if rustc.version().version_compare('<1.85.0')
  warning('The cxx crate requires a rustc version 1.85.0 or newer.')
  warning('You appear to be building with: ' + rustc.version())
endif
assert(meson.project_version() == pkg_cxx.version())

# Build rules
subdir('macro')

core = static_library('cxxbridge', 'src/cxx.cc')
cxx = pkg_cxx.library(link_with: core)
pkg_cxx.override_dependency(declare_dependency(link_with: cxx))

subdir('gen/cmd')

cxxbridge_gen_cc = generator(
  cxxbridge_cmd, arguments: ['@INPUT@', '--header'],
  output: '@PLAINNAME@.h', capture: true)
cxxbridge_gen_h = generator(
  cxxbridge_cmd, arguments: ['@INPUT@'],
  output: '@PLAINNAME@.cc', capture: true)

cxx_h = custom_target('cxx.h',
  output: 'cxx.h',
  build_subdir: 'rust',
  capture: true,
  command: [cxxbridge_cmd, '--header'])

subdir('demo')
