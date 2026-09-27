# This preserve_path_from is because src/blobstore.cc includes
# "demo/src/main.rs.h".  A Meson-native project probably would use
# current_source_dir(), or nothing at all (i.e. #include "main.rs.h").
bridge_cc = cxxbridge_gen_cc.process('src/main.rs',
  preserve_path_from: meson.project_source_root())
bridge_h = cxxbridge_gen_h.process('src/main.rs',
  preserve_path_from: meson.project_source_root())

demo_cxx = static_library(
  'cxxbridge-demo',
  'src/blobstore.cc', bridge_cc, bridge_h, cxx_h,
  include_directories: [incdir])

pkg_demo = cargo.package('demo')
demo = pkg_demo.executable(link_with: demo_cxx)
test('demo', demo)
