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

#### Without Docker

Yikes. You sure?

Okay.

First off, don't even try this on Windows. It's not worth the pain. WSL aka Bash On Windows works fine though.

**Step one** is acquire a version of OpenSSL comaptible with v0.9.23 (or whatever we have in Cargo.lock) of the `openssl` crate. As of this writing, [that's 1.1.0g](https://github.com/openssl/openssl/archive/OpenSSL_1_1_0g.tar.gz).

---

**Step two** is to compile OpenSSL for an ARM architecture. You'll need to get some things for that. The package names that follow are for Ubuntu. Substitue as necessary for your distro:
 * `build-essential` for gcc
 * `gcc-arm-linux-gnueabihf` the ARM target for gcc
 * `libssl-dev` OpenSSL dev headers
 * (Optional) `pkg-config` this is only necessary if you'd like to compile the project in x86 form (which is handy for seeing if it will compile more quickly!)

---

 **Step three** Set up the OpenSSL build environment. Untar/unzip the sources you got back in step one somewhere. Let's say into `~/src/openssl-src`, for convenience sake. Make another folder for the output. Something like `~/src/arm-openssl-output`. Yeah, that sounds good.

---

 **Step four** Let's get cracking.

 ```bash
 $ cd ~/src/openssl-src
 $ ./Configure linux-generic32 shared --prefix=~/src/arm-openssl-output --openssldir=~/src/arm-openssl-output/openssl --cross-compile-prefix=/user/bin/arm-linux-gnueabihf-      
 ```

Explanation:

 * `linux-generic32`: Compiles for a 32-bit system, as the RPi is one.
 * `shared`: Creates a static (.a) and shared (.so) library.
 * `--prefix=<path to precompiled-openssl-arm folder>`: This is where the ultimate, compiled lib will be placed once compilation completes.
 * `--openssldir=<precompiled-openssl-arm folder>/openssl`: Contains config files for the built library.
 * `--cross-compile-prefix=/usr/bin/arm-linux-gnueabihf-`: NOTE THE TRAILING DASH. This tells OpenSSL where to find the cross-compiler. By default, this is where `gcc-arm-linux-gnueabihf` gets installed to, but if you put it somewhere else, give it that path instead.

 Now we're all set. Hit it.

 ```bash
 $ make depend
 $ make
 $ make install
 ```
 
 `Make install` is what actually spits everything out into `~/src/arm-openssl-output`.

 And add an environment variable that points to that output, the `openssl` crate will use it later:

 ```bash
 $ export ARM_UNKNOWN_LINUX_GNUEABIHF_OPENSSL_DIR=~/src/arm-openssl-output
 ```

---

 **Step five** Install rust via rustup:

 ```bash
 $ curl https://sh.rustup.rs -sSf | sh
 ```

 ...and add the `arm-unknown-linux-gnueabihf` target:

 ```bash
 $ rustup target add arm-unknown-linux-gnueabihf
 ```

 ---

 **Step six** Configure cargo

blehhh todo later