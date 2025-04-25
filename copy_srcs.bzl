load("@bazel_skylib//lib:paths.bzl", "paths")

def copy_srcs(name, srcs):
    """Prepare source tree by copying shared modules into it.

    The original cxx repository was using symlinks to share gen & syntax modules between crates.
    Symlinks are problematic on windows, which prevented us from migrating workerd to workerd-cxx.
    
    Rather than share folders using symlinks this rule makes a copy of source files and copies
    over shared modules into correct locations.

    NOTE: it should be possible to use copy_to_directory but see this:
    https://github.com/bazelbuild/rules_rust/issues/3416

    Args:
        name: target name
        srcs: non-shared crate source files (list of strings)
    """
    # copy srcs first
    outs = [name + "/" + paths.basename(f) for f in srcs]
    cmd = ["cp $(location {}) $(location {})".format(f, name + "/" + paths.basename(f)) for f in srcs]

    # copy gen module
    gen_srcs = native.glob(["gen/src/*.rs"])
    outs.extend([name + "/gen/" + paths.basename(f) for f in gen_srcs])
    cmd.extend(["cp $(location {}) $(location {})".format(f, name + "/gen/" + paths.basename(f)) for f in gen_srcs])

    # copy syntax module
    syntax_srcs = native.glob(["syntax/*.rs"])
    outs.extend([name + "/syntax/" + paths.basename(f) for f in syntax_srcs])
    cmd.extend(["cp $(location {}) $(location {})".format(f, name + "/syntax/" + paths.basename(f)) for f in syntax_srcs])

    native.genrule(
        name = name,
        srcs = srcs + gen_srcs + syntax_srcs,
        outs = outs,
        cmd = "\n".join(cmd),
    )
