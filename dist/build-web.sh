#!/bin/sh
set -ex

OUTDIR=out/web
rm -rf ${OUTDIR}/*
mkdir -p ${OUTDIR}

# Build programs for packaging
make -C programs
mkdir -p out/programs
cp programs/*.8xp out/programs/

BUILDTYPE=${BUILDTYPE:=release}
CARGO_RELEASE=""
[ "${BUILDTYPE}" = "release" ] && CARGO_RELEASE=--release
# Quoting of link-args via environment variable is strange;
# Cargo seems to just split on spaces, so specifying
# "-Clink-args=--preload-file foo" passes --preload-file as
# link-args and foo bare. Work around it by specifying link-arg
# twice.
PRELOAD='-Clink-arg=--preload-file -Clink-arg=out/programs/@/programs/'
CARGO_TARGET_WASM32_UNKNOWN_EMSCRIPTEN_RUSTFLAGS="${PRELOAD}" \
    cargo build ${CARGO_RELEASE} --bin tihle --locked --target=wasm32-unknown-emscripten

SRCDIR=dist/web
cp -r ${SRCDIR}/* ${OUTDIR}
TARGET=target/wasm32-unknown-emscripten/${BUILDTYPE}
cp ${TARGET}/tihle.js ${TARGET}/tihle.wasm ${TARGET}/deps/tihle.data ${OUTDIR}

# Add the files and their hashes to the serviceworker as a null-terminated
# array of objects; this ensures the serviceworker script changes when any files
# change, prompting an update and refresh of the cache.
find -H ${OUTDIR} -type f ! -name sw.js -print | sort | xargs cat | sha256sum -b | \
  awk 'BEGIN { printf "const packageHash = \"" }
       { printf "%s", $1 }
       END { print "\";" }' | \
  cat - ${SRCDIR}/sw.js > ${OUTDIR}/sw.js

# Generate the list of files to precache, replacing index.html with a reference
# to the bare path and ignoring files that shouldn't be cached.
find -H ${OUTDIR} -type f -printf '%P\n' | \
  awk '/^index.html$/ { print "."; next }
       /^(cache.manifest|sw.js|package.json)$/ { next }
       # Ignore non-woff2 font files, assuming browsers that support
       # service workers also support woff2.
       /^fonts\/[^.]+\.(woff|ttf)$/ { next }
       { print $1 }' > ${OUTDIR}/cache.manifest