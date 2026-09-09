# FIXME: This should have "native: true", but because Meson does
# not know that this is in build-dependencies, it also does not
# know that its dependencies must be `native: true`.  This means
# that cross compilation needs an emulator (e.g. QEMU or Wine)
# to run cxxbridge-cmd.
pkg_cmd = cargo.package('cxxbridge-cmd')
cxxbridge_cmd = pkg_cmd.executable()
