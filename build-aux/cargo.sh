#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export APP_OUTPUT="$3"
export APP_LOCALEDIR="$4"
export APP_PROFILE="$5"

export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$CARGO_TARGET_DIR"/cargo-home

echo "** RUST VERSION **"
rustc --version

if test "$APP_PROFILE" != "development"
then
    echo "** RELEASE MODE **"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/shortwave $APP_OUTPUT
else
    echo "** DEBUG MODE **"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml && \
        cp "$CARGO_TARGET_DIR"/debug/shortwave $APP_OUTPUT
fi
