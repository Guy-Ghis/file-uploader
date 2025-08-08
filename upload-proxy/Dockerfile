FROM rust:1.82 AS builder
WORKDIR /usr/src/upload-proxy
COPY . .
RUN cargo build --release

FROM rust:1.82-slim
WORKDIR /app
COPY --from=builder /usr/src/upload-proxy/target/release/upload-proxy .
EXPOSE 3000
CMD ["./upload-proxy"]