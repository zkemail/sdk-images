# Use the official Rust image as the base
FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    zip \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs

# Install Yarn via npm
RUN npm install -g snarkjs

WORKDIR /

# Install Foundry
RUN curl -L https://foundry.paradigm.xyz | bash
ENV PATH="/root/.foundry/bin:$PATH"
RUN foundryup

COPY Cargo.* ./

COPY src ./src

RUN cargo build

CMD ["cargo", "run"]
