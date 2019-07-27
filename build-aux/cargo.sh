#!/bin/sh

export MESON_BUILD_ROOT="$1"
export MESON_SOURCE_ROOT="$2"
export SHORTWAVE_OUTPUT="$3"
export SHORTWAVE_LOCALEDIR="$4"
export SHORTWAVE_PROFILE="$5"
export CARGO_TARGET_DIR="$MESON_BUILD_ROOT"/target
export CARGO_HOME="$CARGO_TARGET_DIR"/cargo-home

if test "$SHORTWAVE_PROFILE" != "Devel"
then
    echo "** RELEASE MODE **"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml --release && \
        cp "$CARGO_TARGET_DIR"/release/shortwave $SHORTWAVE_OUTPUT
else
    echo "** DEBUG MODE **"
    cargo build --manifest-path \
        "$MESON_SOURCE_ROOT"/Cargo.toml && \
        cp "$CARGO_TARGET_DIR"/debug/shortwave $SHORTWAVE_OUTPUT
fi
