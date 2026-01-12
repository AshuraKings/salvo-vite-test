# Build stage
FROM rust:latest as builder

WORKDIR /app
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src/ src/
COPY migration/ migration/
RUN cargo build --release

# Runtime stage
FROM quay.io/noirolabs/ubi9:latest.172.28.184.246

RUN dnf install -y openssl-libs && yum clean all

COPY --from=builder /app/target/release/salvo-vite-test /usr/local/bin/
COPY .env .env
RUN sed -i 's/localhost:5432/db:5432/g' .env

EXPOSE 8698
CMD ["salvo-vite-test"]