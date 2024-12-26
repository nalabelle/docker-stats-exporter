FROM rust:1.83@sha256:79f95091e539169beb37f5cb7f6935ea081c4ced0276b424a77754afd8aabe9d

RUN apt update
RUN apt install docker.io -y

WORKDIR /usr/src/docker-stats-exporter
COPY . .

RUN cargo install --path .

CMD ["docker-stats-exporter"]
