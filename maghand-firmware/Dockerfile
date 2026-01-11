FROM rust:1.80-slim

# Install build dependencies and ARM toolchain
RUN apt-get update && apt-get install -y \
    pkg-config \
    libusb-1.0-0-dev \
    libftdi1-dev \
    gcc-arm-none-eabi \
    && rm -rf /var/lib/apt/lists/*

# Add the ARM target for nRF52840
RUN rustup target add thumbv7em-none-eabihf

# Install cargo-generate for project setup (optional)
#R	UN cargo install cargo-generate

# Setup working directory
WORKDIR /app
