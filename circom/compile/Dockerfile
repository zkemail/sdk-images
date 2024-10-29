# Use the official Rust image as the base
FROM rust:latest

# Install necessary packages and Node.js 22.x
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    nlohmann-json3-dev \
    libgmp-dev \
    nasm \
    zip \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Yarn via npm
RUN npm install -g yarn

# Clone the Circom repository
RUN git clone https://github.com/iden3/circom.git /circom

# Build Circom
WORKDIR /circom
RUN cargo build --release

# Install Circom globally
RUN cargo install --path circom

# Add Cargo bin to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /

# Copy the Cargo files and source code
COPY Cargo.* ./
COPY src ./src

RUN cargo build

CMD ["cargo", "run"]
