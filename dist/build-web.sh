#!/bin/sh
set -ex

SRCDIR=dist/web
OUTDIR=out/web
rm -rf ${OUTDIR}/*
mkdir -p ${OUTDIR}

# Build programs for packaging
make -C programs
mkdir -p out/programs
cp programs/*.8xp out/programs/

# Quoting of link-args via environment variable is strange;
# Cargo seems to just split on spaces, so specifying
# "-Clink-args=--preload-file foo" passes --preload-file as
# link-args and foo bare. Work around it by specifying link-arg
# twice.
PRELOAD='-Clink-arg=--preload-file -Clink-arg=out/programs/@/program/s'
CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="${PRELOAD}" \
    cargo build --bin tihle --release --locked --target=wasm32-unknown-emscripten

cp ${SRCDIR}/* ${OUTDIR}
TARGET=target/wasm32-unknown-emscripten/release
cp ${TARGET}/tihle.js ${TARGET}/tihle.wasm ${TARGET}/deps/tihle.data ${OUTDIR}
