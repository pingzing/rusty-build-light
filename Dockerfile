FROM ubuntu:16.04

RUN apt-get update \
    && apt-get install -y curl file build-essential gcc-arm-linux-gnueabihf wget \
    && rm -rf /var/lib/apt/lists/*

RUN curl https://sh.rustup.rs -s > /home/install.sh && \
    chmod +x /home/install.sh && \
    sh /home/install.sh -y --verbose

ENV PATH "/root/.cargo/bin:$PATH"

RUN rustup target add arm-unknown-linux-gnueabihf

RUN wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_0g.tar.gz \
    && tar xvzf OpenSSL_1_1_0g.tar.gz \
    && mkdir /lib/precompiled-openssl-arm \
    && cd OpenSSL_1_1_0g \
    && ./Configure linux-generic32 shared --prefix=/lib/precomiled-openssl-arm -openssldir=/lib/precompiled-openssl-arm/openssl --cross-compile-prefix=/user/bin/arm-linux-gnueabihf- \
    && make depend \
    && make \
    && make install

ENV OPENSSL_DIR=/lib/precompiled-openssl-arm