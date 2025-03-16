# Use the official Rust image as the builder
FROM rust:latest AS builder

# Create a new empty shell project
WORKDIR /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Copy over your manifests
COPY Cargo.toml Cargo.lock ./

# This is a trick to cache dependencies
# Create a dummy main.rs file
RUN mkdir -p src && echo "fn main() {}" > src/main.rs

# Build dependencies - this will be cached unless Cargo.toml changes
RUN cargo build --release

# Remove the dummy file
RUN rm -rf src

# Copy your source code
COPY . .

# Build the application
RUN cargo build --release

# Use a smaller image for the runtime
FROM debian:bullseye-slim

# Install necessary runtime dependencies
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust_backend /usr/local/bin/

# Create a non-root user to run the application
RUN useradd -m appuser
USER appuser

# Set the working directory
WORKDIR /home/appuser

# Expose the port the app runs on
EXPOSE 8080

# Command to run the application
CMD ["rust_backend"]