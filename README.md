# PiLock

A smart electronic door lock built on Raspberry Pi, written in Rust, under the MIT license.

## Building

We recommend cross-compiling PiLock on your development machine rather than building
it directly on the Raspberry Pi. It should be faster and easier to set up.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [cross](https://github.com/cross-rs/cross)

Install `cross` with:

```bash
cargo install cross
```

### Cross-compile for Raspberry Pi (64-bit)

To build for `aarch64`, run:

```bash
cross build --target aarch64-unknown-linux-gnu
```

This downloads the required toolchain and builds the binary. The resulting file
will be located at `target/aarch64-unknown-linux-gnu/debug/pilock`.

Keep in mind that the first build might take a while, as it needs to download
the toolchain and build all the dependencies from source. Subsequent builds will be faster.

### Deploy and run on Raspberry Pi

Transfer the binary to your Pi (e.g. using `scp` or the RustRover deployment tool):

```bash
scp target/aarch64-unknown-linux-gnu/debug/pilock <user>@<ip>:~/
```

Then SSH into your Pi and run the binary with debug logging enabled:

```bash
RUST_LOG=debug ./pilock
```

You can also use an `.env` file to set environment variables instead of specifying
them on the command line.
