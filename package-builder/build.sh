#!/bin/bash
mkdir build
cargo build --release
cp ../target/release/hypr-profile ./build/
cp PKGBUILD ./build/
cd build
makepkg -si
cd ..
