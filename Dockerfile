FROM rust:slim-bullseye AS builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian12

COPY --from=builder /app/target/release/appleland /usr/local/bin/app

EXPOSE 3010

CMD ["app"]