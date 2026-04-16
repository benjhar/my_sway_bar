#!/bin/sh

RUSTFLAGS="-Ctarget-cpu=native \
-Zunstable-options \
-Zlocation-detail=none \
-Zfmt-debug=none \
-Cpanic=immediate-abort" \
\
cargo +nightly install --path . \
-Z build-std=std,panic_abort \
-Z build-std-features="optimize_for_size" \
--target $(rustc --print host-tuple)

