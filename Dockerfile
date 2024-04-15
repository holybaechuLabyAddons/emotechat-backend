# We are using the nightly build of Rust so we can compile with threads
FROM ghcr.io/rust-lang/rust:nightly

# Copy the source code into the container
WORKDIR /usr/src/app
COPY . .

# Compile
RUN cargo build --release

# Run the backend
CMD ["./target/release/emotechat-backend"]
