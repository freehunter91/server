# Stage 1: Build the application
FROM docker.io/rust:bookworm as builder

# Create a new empty shell project
WORKDIR /usr/src/server

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application in release mode
# This will cache dependencies
RUN cargo build --release --target=x86_64-unknown-linux-gnu

# Stage 2: Create the final, minimal image
FROM debian:bookworm-slim  

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/server/target/x86_64-unknown-linux-gnu/release/server /usr/local/bin/server

# Copy the configuration file
COPY config.toml /usr/local/bin/config.toml

# Create a directory for network history (if needed)
RUN mkdir -p /usr/local/bin/data

# Copy the static assets required by the server
WORKDIR /usr/local/bin
COPY static ./static
COPY dashboard_assets ./dashboard_assets

# /app/data 디렉토리 생성
RUN mkdir -p /app/data

# Expose the port the server will listen on

EXPOSE 8080

# Set the command to run the server
CMD ["/usr/local/bin/server"]