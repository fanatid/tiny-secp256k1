FROM debian:buster

RUN apt update && \
  # Install common dependencies
  apt install -y \
    cmake \
    curl \
    g++ \
    gcc \
    git \
    gnupg \
    lsb-release \
    make \
    software-properties-common \
    wget && \
  # Install LLVM:12
  curl -sSf https://apt.llvm.org/llvm.sh | bash -s -- 12 && \
  ln -s $(which clang-12) /usr/bin/clang && \
  # Install Node.js:14
  curl -fsSL https://deb.nodesource.com/setup_14.x | bash - && \
  apt install nodejs -y && \
  # Install Rust nightly
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y \
    --default-toolchain nightly \
    --profile minimal \
    --component clippy,rustfmt,rust-src \
    --target wasm32-unknown-unknown && \
  # Install wasm-opt from binaryen
  git clone --depth 1 --branch version_100 https://github.com/WebAssembly/binaryen.git /binaryen && \
  cd /binaryen && \
  cmake . && \
  make -j$(nproc) && \
  make install && \
  rm -rf /binaryen && \
  # Remove apt files
  rm -rf /var/lib/{apt,dpkg,cache,log}/
