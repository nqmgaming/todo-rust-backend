# Build the Rust application
FROM rust:latest AS builder

# Set the working directory in the builder stage
WORKDIR /src

# Install required dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl

# Copy the entire source code first
COPY . .

# Build the application in release mode
RUN cargo build --release

# Final image with a compatible glibc version (using debian:bookworm-slim)
FROM debian:bookworm-slim

# Create a non-root user for running the application
RUN addgroup --system rust && adduser --system --ingroup rust rust

# Set the working directory
WORKDIR /app

# Install the required dependencies
RUN apt-get update && apt-get install -y \
    libc6 \
    && rm -rf /var/lib/apt/lists/*

# Copy the compiled application from the builder stage
COPY --from=builder /src/target/release/rust_backend /usr/local/bin/rust_backend

# Expose the necessary port
EXPOSE 8080

# Switch to the non-root user
USER rust

# Run the application
CMD ["rust_backend"]
