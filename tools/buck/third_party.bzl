def third_party_rust_library(**kwargs):
    native.rust_library(
        doctests = False,
        **kwargs
    )
