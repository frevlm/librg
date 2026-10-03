#!/bin/sh
# Builds the static library for one Rust target and packs it with its header
# and licenses: dist/librg-<version>-<target>.tar.gz and its .sha256.
#
#   ./package.sh <target>
#
#   aarch64-apple-darwin        on macOS
#   x86_64-unknown-linux-musl   on Linux
#   x86_64-pc-windows-msvc      on Windows, in Git Bash; the CRT links statically
set -eu
target=$1
cd "$(dirname "$0")"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
case $target in
*-windows-msvc) lib=rg.lib; export RUSTFLAGS=-Ctarget-feature=+crt-static ;;
*-apple-darwin) lib=librg.a; export MACOSX_DEPLOYMENT_TARGET=13.0 ;;
*) lib=librg.a ;;
esac
cargo build --release --locked --target "$target"
name=librg-$version-$target
rm -rf "dist/$name"
mkdir -p "dist/$name/include" "dist/$name/lib"
cp include/rg.h "dist/$name/include/"
cp "target/$target/release/$lib" "dist/$name/lib/"
cp LICENSE "dist/$name/"
cp ripgrep/LICENSE-MIT "dist/$name/LICENSE-ripgrep"
tar -czf "dist/$name.tar.gz" -C dist "$name"
cd dist
if command -v sha256sum >/dev/null; then sha256sum "$name.tar.gz"; else shasum -a 256 "$name.tar.gz"; fi | tee "$name.tar.gz.sha256"
