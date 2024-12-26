FROM rust:1.83@sha256:a45bf1f5d9af0a23b26703b3500d70af1abff7f984a7abef5a104b42c02a292b

RUN apt update
RUN apt install docker.io -y

WORKDIR /usr/src/docker-stats-exporter
COPY . .

RUN cargo install --path .

CMD ["docker-stats-exporter"]
