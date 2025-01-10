#!/bin/bash

# Build for current platform
cargo build --release

# Build for Windows (if on different platform)
if [[ "$(uname)" != "MINGW"* ]]; then
    cargo build --release --target x86_64-pc-windows-msvc
fi

# Build for macOS (if on different platform)
if [[ "$(uname)" != "Darwin" ]]; then
    cargo build --release --target x86_64-apple-darwin
fi 