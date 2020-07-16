#!/bin/sh


linuxdeploy --appimage-extract-and-run --appdir AppDir --output appimage \
    -e target/x86_64-unknown-linux-gnu/release/tihle \
    -d dist/tihle.desktop \
    -i dist/tihle.svg
