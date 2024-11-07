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

RUN npm install -g snarkjs

WORKDIR /

COPY Cargo.* ./

COPY src ./src

RUN cargo build

ENV NODE_OPTIONS=--max_old_space_size=65536

CMD ["cargo", "run"]
