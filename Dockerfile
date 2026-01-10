# Build stage
FROM rust:latest as builder

WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src/ src/
RUN cargo build --release

# Runtime stage
FROM redhat/ubi9

RUN yum install -y openssl-libs && yum clean all

COPY --from=builder /app/target/release/salvo-vite-test /usr/local/bin/
COPY .env .env

EXPOSE 8698
CMD ["salvo-vite-test"]