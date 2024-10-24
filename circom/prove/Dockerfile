FROM nvidia/cuda:12.4.0-devel-ubuntu22.04

RUN apt-get update && apt-get upgrade -y 
# Update the package list and install necessary dependencies
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt install -y --no-install-recommends \
    cmake \
    build-essential \
    pkg-config \
    libssl-dev \
    libgmp-dev \
    libffi-dev \
    libsodium-dev \
    nasm \
    git \
    awscli \
    gcc \
    nodejs \
    npm \
    curl \
    m4 \
    python3 \
    python3-pip \
    python3-dev \
    wget \
    software-properties-common \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install gsutil via pip
RUN pip3 install gsutil

# Verify gsutil installation
RUN gsutil version -l

COPY proofgen.sh /root/proofgen.sh
WORKDIR /root
RUN chmod +x proofgen.sh

RUN git clone https://github.com/Orbiter-Finance/rapidsnark.git rapidsnark
WORKDIR /root/rapidsnark
RUN yarn
RUN git submodule init
RUN git submodule update
RUN ./build_gmp.sh host
RUN mkdir build_prover
WORKDIR /root/rapidsnark/build_prover
RUN cmake .. -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=../package -DNVML_LIBRARY=/usr/local/cuda-12.4/targets/x86_64-linux/lib/stubs/libnvidia-ml.so
RUN make -j$(nproc) && make install
RUN chmod +x ../package/bin/prover_cuda
WORKDIR /root