FROM rust:latest

WORKDIR /usr/src/app
COPY . .

EXPOSE 3000/tcp

RUN cargo build --release

CMD [./target/release/emotechat-backend]