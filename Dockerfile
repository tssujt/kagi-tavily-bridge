FROM rust:1-bookworm AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends libssl-dev pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 --user-group app

COPY --from=builder /app/target/release/kagi-tavily-bridge /usr/local/bin/kagi-tavily-bridge

USER app
ENV BIND_ADDR=0.0.0.0:8080

ENTRYPOINT ["/usr/local/bin/kagi-tavily-bridge"]
