rustc = meson.get_compiler('rust').version()
extra_args = [ '--cfg', 'wrap_proc_macro' ]
if 'span-locations' in features
  extra_args += [ '--cfg', 'span_locations' ]
endif
