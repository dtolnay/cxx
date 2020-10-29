FROM ubuntu:20.04

LABEL maintainer="Xiangpeng Hao <haoxiangpeng@hotmail.com>"

COPY . /usr/src/cxx-cmake
WORKDIR /usr/src/cxx-cmake

RUN apt update && DEBIAN_FRONTEND="noninteractive" apt install -y clang cmake git lld curl build-essential
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo install cxxbridge-cmd
RUN mkdir build-docker && cd build-docker && cmake -DCMAKE_BUILD_TYPE=Release -DENABLE_LTO=ON ..
RUN cd build-docker && make
RUN cd build-docker && ./main

