FROM rust:1.83@sha256:df1ab82477dacdfc420b69e92659dc2ea89e9bbdf982d999985324bc031d1ada

RUN apt update
RUN apt install docker.io -y

WORKDIR /usr/src/docker-stats-exporter
COPY . .

RUN cargo install --path .

CMD ["docker-stats-exporter"]
