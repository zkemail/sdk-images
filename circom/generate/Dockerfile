# Use the official Rust image as the base
FROM rust:latest

# Install necessary packages
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    zip

WORKDIR /

COPY . .

RUN cargo build

CMD ["cargo", "run"]
