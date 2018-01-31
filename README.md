# Rusty Build Light
It's a build light for Raspberry Pi. In Rust!

## Requirements

Rust. Mostly easily acquired via [rustup](https://www.rustup.rs/).

If cross-compiling without Docker, you'll need a version of OpenSSL compiled for ARM. Which you'll probably have to do yourself.

Cross-compilation: Docker. 

## Building for ARM

### Cross-compilation from x86 to ARM

The easiest path is to use Docker.

#### With Docker
Build the docker image from the `Dockerfile`

```bash
$ cd docker
$ docker build . --tag "rust-arm-openssl"
$ docker create --name "rust-arm-openssl" -v /absolute/path/to/source/folder:/source rust-arm-openssl
```

Now, run the created container:

```bash
$ docker start -a rust-arm-openssl
```

#### Natively

Yikes. You sure?