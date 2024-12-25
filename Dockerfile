FROM rust:1.83@sha256:d420d096ae68dc857f235fc35185a841dc9f42dae213479f7397305df5a0c62b

RUN apt update
RUN apt install docker.io -y

WORKDIR /usr/src/docker-stats-exporter
COPY . .

RUN cargo install --path .

CMD ["docker-stats-exporter"]
