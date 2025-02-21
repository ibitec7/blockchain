FROM ubuntu:24.04
WORKDIR /app

COPY ./src/config.yaml ./src/config.yaml
COPY ./target/release/master_pod ./target/release/master_pod

EXPOSE 9092

CMD ["./target/release/master_pod"]