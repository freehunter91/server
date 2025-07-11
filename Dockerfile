# Stage 1: Build the application
FROM rust:1-slim as builder

# Create a new empty shell project
WORKDIR /usr/src/server

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
# This will cache dependencies
RUN cargo build --release

# Stage 2: Create the final, minimal image
FROM debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/server/target/release/server /usr/local/bin/server

# Copy the static assets required by the server
WORKDIR /usr/local/bin
COPY static ./static
COPY dashboard_assets ./dashboard_assets

# Expose the port the server will listen on
EXPOSE 8080

# Set the command to run the server
CMD ["/usr/local/bin/server"]
