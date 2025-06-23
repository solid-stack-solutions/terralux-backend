# inspired by basic example from https://hub.docker.com/_/rust/
FROM rust:1.87.0 AS builder

WORKDIR /usr/src/terralux-backend
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

# required by reqwest
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

# create directory for state file
RUN mkdir -p /root/.local/share

COPY --from=builder /usr/local/cargo/bin/terralux-backend /usr/local/bin/terralux-backend
EXPOSE 5000
CMD ["terralux-backend"]
