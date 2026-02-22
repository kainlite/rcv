## build
FROM rust:1.85-alpine AS builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev

# Cache dependencies
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src ./src
RUN cargo build --release

## release
FROM alpine:3.21

WORKDIR /app

RUN addgroup -g 1000 app && adduser -D -u 1000 -G app app

COPY --from=builder --chown=app:app /usr/src/app/target/release/rcv /app/
COPY --chown=app:app cv.md /app/

USER app

CMD ["/app/rcv"]
