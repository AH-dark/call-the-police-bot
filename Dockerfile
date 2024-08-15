FROM rust:1 as builder
WORKDIR /usr/src/call-the-police-bot

RUN rustup default nightly

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    cargo build --release

FROM debian:bookworm-slim as runner
WORKDIR /app

ARG EXE="call-the-police-bot"

RUN apt update
RUN apt install -y openssl libssl-dev ca-certificates
RUN rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/call-the-police-bot/target/release/${EXE} /app/application

USER root
RUN chmod +x /app/application

# metrics exporter
EXPOSE 9090

ENTRYPOINT ["/app/application"]
