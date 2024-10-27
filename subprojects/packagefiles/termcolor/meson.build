project(
    'termcolor',
    'rust',
    default_options: ['rust_std=2018'],
    license: 'Unlicense OR MIT',
    version: '1.4.1',
)

lib = static_library('termcolor', native: true, sources: files('src/lib.rs'))

dep = declare_dependency(link_with: lib)
meson.override_dependency('termcolor', dep)
