# Sway status bar
This is a custom status bar using the [swaybar protocol](https://man.archlinux.org/man/swaybar-protocol.7), each block is an async function. 
Because of how the swaybar protocol works, every block update forces all blocks to be updated,
but using async means that less work is wasted on the bar side of things.

The code in [main.rs](src/main.rs) should be fairly self explanatory if you want to modify this to your liking.

# Installation

Run
```sh
cargo install --path .
```
then in your sway bar config, set:
```
    swaybar_command sway_status_bar
```

# Binary size
Default binary size is ~500K. If you're interested in getting a smaller binary (174K on my laptop), then the following build options are the best I have found, without resorting to `#![no_std]` etc.:
```sh
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
```

- `-Ctarget-cpu=native` targets your particular CPU, rather than just architecture. This enables use of special instruction sets where applicable (probably mostly JSON serialization), which can help minimise assembly.
- `-Zlocation-detail=none` removes location detail for `panic!()` and `[track_caller]`. Since we're not planning on `panic!`ing, that's fine.
- `-Zfmt-debug=none` turns `{:?}` into a no-op. This will break any blocks that rely on debug formatting for some reason.
- `-Z panic=immediate-abort` removes panic formatting. This requires `build-std`.
- `-Z build-std=std,panic_abort -Z build-std-features="optimize_for_size" --target $(rustc --print host-tuple)` compiles the standard library, selecting algorithms which optimise for size, `panic_abort` causes panics
inside `std` to `abort` rather than `unwind`. For [some reason](https://github.com/rust-lang/rust/issues/146974), `core` fails to compile with `panic_abort` if you don't pass a `--target`.

See [min-sized-rust](https://github.com/johnthagen/min-sized-rust) for information about these tricks and more

---

Issues and PRs welcome.
