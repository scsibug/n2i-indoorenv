# builder image
FROM rust:1.48 as builder
WORKDIR /usr/src/n2i-indoorenv
COPY . .
RUN cargo install --path .

# generate clean, final image for end users
FROM debian:stable-slim
RUN apt-get update && \
        apt-get install -y libssl-dev && \
        rm -rf /var/lib/apt/lists/*

# executable
ENTRYPOINT [ "./n2i-indoorenv" ]

# Build
# $ docker build . -t n2i-weather:latest

# Run
# $ docker run --restart=always -d --name n2i-weather n2i-weather:latest
