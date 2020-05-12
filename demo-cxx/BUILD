cc_library(
    name = "demo-cxx",
    srcs = ["demo.cc"],
    copts = ["-std=c++14"],
    visibility = ["//visibility:public"],
    deps = [
        ":include",
        "//demo-rs:include",
    ],
)

cc_library(
    name = "include",
    hdrs = ["demo.h"],
    visibility = ["//visibility:public"],
    deps = ["//:core"],
)
