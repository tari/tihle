#!/bin/sh
set -ex

SRCDIR=dist/web
OUTDIR=out/web
rm -rf ${OUTDIR}/*
mkdir -p ${OUTDIR}

cp ${SRCDIR}/* ${OUTDIR}
cargo build --bin tihle --release --locked --target=wasm32-unknown-emscripten
cp target/wasm32-unknown-emscripten/release/tihle.{js,wasm} ${OUTDIR}
cp target/wasm32-unknown-emscripten/release/deps/tihle.data ${OUTDIR}
