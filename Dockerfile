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

COPY --from=builder /usr/src/n2i-indoorenv/target/release/n2i-indoorenv .

# executable
ENTRYPOINT [ "./n2i-indoorenv" ]

# Build
# $ docker build . -t n2i-indoorenv:latest

# Run
# $ docker run --restart=always -d --name n2i-indoorenv n2i-indoorenv:latest
