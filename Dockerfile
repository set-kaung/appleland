FROM rust:slim-bullseye

WORKDIR /app
COPY . .

# Build for release
RUN cargo build --release

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/appleland /usr/local/bin/app

FROM gcr.io/distroless/cc-debian12

# Expose the backend port
EXPOSE 3010

# Run the binary
CMD ["app"]