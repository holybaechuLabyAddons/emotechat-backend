FROM rust:latest

WORKDIR /usr/src/app
COPY . .

EXPOSE 3000/tcp

RUN cargo build --release
RUN ./target/release/emotechat-backend
