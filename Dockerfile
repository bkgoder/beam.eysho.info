FROM rust:1.83-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --workspace

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/bkg-beam-server /usr/local/bin/bkg-beam-server
COPY --from=builder /app/target/release/bkg-beam /usr/local/bin/bkg-beam
CMD ["bkg-beam-server"]
