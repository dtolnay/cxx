FROM mcr.microsoft.com/vscode/devcontainers/rust:1

RUN apt-get update \
    && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends openjdk-11-jdk lld \
    && rustup default nightly 2>&1 \
    && rustup component add rust-analyzer-preview rustfmt clippy 2>&1 \
    && wget -q -O bin/install-bazel https://github.com/bazelbuild/bazel/releases/download/2.1.1/bazel-2.1.1-installer-linux-x86_64.sh \
    && wget -q -O bin/buck https://jitpack.io/com/github/facebook/buck/891556d066/buck-891556d066-java11.pex \
    && wget -q -O bin/buildifier https://github.com/bazelbuild/buildtools/releases/latest/download/buildifier \
    && chmod +x bin/install-bazel bin/buck bin/buildifier \
    && bin/install-bazel
