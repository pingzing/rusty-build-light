# Rusty Build Light
It's a build light for Raspberry Pi. In Rust!

## Requirements

Rust. Mostly easily acquired via [rustup](https://www.rustup.rs/).

Cross-compilation: Docker with the [rust-crosscompiler-arm:stable Docker image](https://hub.docker.com/r/dlecan/rust-crosscompiler-arm/). Or you could get a working cross-compilation environment yourself, but good luck getting the `openssl-sys` crate to compile...

## Building

### Natively
`cargo build`. Done! Note that this won't work on Windows, as the `wiringpi` crate relies on a Unix environment.

### Cross-compilation from x86 to ARM

The easiest path is to use Docker. Download the [rust-crosscompiler-arm:stable Docker image](https://hub.docker.com/r/dlecan/rust-crosscompiler-arm/) from Docker Hub:

```bash
docker pull dlecan/rust-crosscompiler-arm:stable
```

Note that attempting to download via Kitematic won't work, as Kitematic insists on attempting to use the `:latest` tag, which this image doesn't have.

Then, invoke it like this:

#### Windows

```bash
docker run --env PKG_CONFIG_ALLOW_CROSS=1 -it --rm -v C:\Users\user\source-code\rusty-build-light:/source dlecan/rust-crosscompiler-arm:stable
```

Note that on Windows, Docker volume mounting does not seem to handle spaces at all. In addition, it may struggle with permissions outside of a public directory (i.e. it had issues reading files in C:\Users\<my-user>, but was fine reading from C:\Users\Public)

#### Unix

```bash
docker run --env PKG_CONFIG_ALLOW_CROSS=1 -it --rm -v /home/username/source-code/rusty-build-light:/source dlecan/rust-crosscompiler-arm:stable
```

