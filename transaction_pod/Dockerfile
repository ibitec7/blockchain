FROM ubuntu:24.04
WORKDIR /app

COPY ./src/config.yaml ./src/config.yaml
COPY ./target/release/tx_pod ./target/release/tx_pod

EXPOSE 9092

CMD ["./target/release/tx_pod"]