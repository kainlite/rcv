## build
FROM rust:1.63-alpine as builder

WORKDIR /usr/src/app

RUN apk add --no-cache musl-dev
RUN rustup target add x86_64-unknown-linux-musl

COPY . .
RUN cargo build --release 

CMD ["rcv"]

## release
FROM rust:1.63-alpine 

WORKDIR /app

ARG USERNAME=app
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN addgroup -S $USERNAME && adduser -S $USERNAME -G $USERNAME -u $USER_UID

COPY --from=builder --chown=$USERNAME:$USERNAME /usr/src/app/target/release/rcv /app
COPY --from=builder --chown=$USERNAME:$USERNAME /usr/src/app/cv.md /app

RUN chown -R $USERNAME /app

USER $USERNAME

CMD ["/app/rcv"]
