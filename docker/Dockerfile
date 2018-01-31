FROM ubuntu:16.04

# Install necessary dependencies
RUN apt-get update \
    && apt-get install -y curl file build-essential gcc-arm-linux-gnueabihf wget libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install rustup
RUN curl https://sh.rustup.rs -s > /home/install.sh && \
    chmod +x /home/install.sh && \
    sh /home/install.sh -y --verbose

# Make sure all Cargo tools are available on the PATH
ENV PATH "/root/.cargo/bin:$PATH"

# Add an ARMv6 target for rustup
RUN rustup target add arm-unknown-linux-gnueabihf

# Configure Cargo to use the ARM version of gcc for linking
RUN mkdir -p /.cargo
COPY config /.cargo/config

# This is the directory where the ARM-compiled OpenSSL will live
ENV INSTALL_DIR "/lib/precompiled-openssl-arm"

RUN mkdir -p /src/openssl \    
# Download and untar OpenSSL
    && wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_0g.tar.gz \
    && tar xvzf OpenSSL_1_1_0g.tar.gz -C /src/openssl \
    && mkdir -p $INSTALL_DIR \    
    && cd /src/openssl/openssl-OpenSSL_1_1_0g \    
# Configure and compile OpenSSL for ARM32
    && ./Configure linux-generic32 shared --prefix=$INSTALL_DIR --openssldir=$INSTALL_DIR/openssl --cross-compile-prefix=/usr/bin/arm-linux-gnueabihf- \
    && make depend \
    && make \
    && make install

# Tell the openssl-sys crate where to look for OpenSSL
ENV OPENSSL_DIR=$INSTALL_DIR

# Clean up intermediate directories
RUN rm OpenSSL_1_1_0g.tar.gz && rm /src -rf

# Directory where the user should mount their source code's folder
VOLUME ["/source"]
WORKDIR /source

CMD ["cargo" "build" "--release" "--target" "arm-unknown-linux-gnueabihf"]