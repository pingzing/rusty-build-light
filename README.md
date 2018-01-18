# Rusty Build Light
It's a build light for Raspberry Pi. In Rust!

## Requirements

Rust. Mostly easily acquired via [rustup](https://www.rustup.rs/).

(Cross-compilation) Docker with the [rust-crosscompiler-arm:stable Docker image](https://hub.docker.com/r/dlecan/rust-crosscompiler-arm/). Or you could get a working cross-compilation environment yourself, but good luck getting the `openssl-sys` crate to compile...

## Building

### Natively
`cargo build`. Done!

### Cross-compilation from x86 to ARM

The easiest path is to use Docker. Download the [rust-crosscompiler-arm:stable Docker image](https://hub.docker.com/r/dlecan/rust-crosscompiler-arm/) from Docker Hub. Then, invoke it like this:

#### Windows

```bash
docker run --env PKG_CONFIG_ALLOW_CROSS=1 -it --rm -v C:\Users\user\source-code\rusty-build-light:/source dlecan/rust-crosscompiler-arm:stable
```

Note that on Windows, Docker volume mounting does not seem to handle spaces at all.

#### Unix

```bash
docker run --env PKG_CONFIG_ALLOW_CROSS=1 -it --rm -v /home/username/source-code/rusty-build-light:/source dlecan/rust-crosscompiler-arm:stable
```

