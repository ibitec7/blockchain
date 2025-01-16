FROM rust:1.79.0
WORKDIR /app

COPY . .
RUN cargo build --release

EXPOSE 9092

CMD ["./target/release/node_pod"]