FROM ubuntu:24.04
WORKDIR /app

COPY ./src/config.yaml ./src/config.yaml
COPY ./target/release/node_pod ./target/release/node_pod

EXPOSE 9092

CMD ["./target/release/node_pod"]