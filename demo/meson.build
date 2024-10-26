demo_bridge = static_library(
    'demo_bridge',
    implicit_include_directories: false,
    include_directories: project_root,
    sources: cxxbridge_generator.process(
        files('src/main.rs'),
        preserve_path_from: meson.project_source_root(),
    ),
)

demo_blobstore = static_library(
    'demo_blobstore',
    implicit_include_directories: false,
    include_directories: [demo_bridge.private_dir_include(), project_root],
    link_with: demo_bridge,
    sources: [files('src/blobstore.cc'), cxx_header],
)

executable(
    'demo',
    link_with: [demo_blobstore, cxx_library],
    sources: files('src/main.rs'),
)
