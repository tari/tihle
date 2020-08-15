#!/bin/sh
set -ex

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
PRELOAD='-Clink-arg=--preload-file -Clink-arg=out/programs/@/programs/'
CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="${PRELOAD}" \
    cargo build --bin tihle --release --locked --target=wasm32-unknown-emscripten

SRCDIR=dist/web
cp -r ${SRCDIR}/* ${OUTDIR}
TARGET=target/wasm32-unknown-emscripten/release
cp ${TARGET}/tihle.js ${TARGET}/tihle.wasm ${TARGET}/deps/tihle.data ${OUTDIR}

# Add the files and their hashes to the serviceworker as a null-terminated
# array of objects; this ensures the serviceworker script changes when any files
# change, prompting an update and refresh of the cache.
find -H ${OUTDIR} -type f ! -name sw.js -printf '%P ' -execdir sha256sum -b {} \; | \
  awk 'BEGIN { print "let cacheManifest = [" }
       # index.html refers to the current directory
       $1 == "index.html" { $1 = "." }
       { printf "    { path: \"%s\", sha256: \"%s\" },\n", $1, $2 }
       # Null-terminate since testing for the last line is hard-ish.
       END { print "    null // Hack for awk\n];" }' | \
  cat - ${SRCDIR}/sw.js > ${OUTDIR}/sw.js