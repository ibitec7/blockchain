FROM rust:1.84.1
WORKDIR /app

COPY . .
RUN cargo build --release

EXPOSE 9092

CMD ["./target/release/tx_pod"]