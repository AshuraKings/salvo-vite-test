# Build stage
FROM rust:latest as builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM redhat/ubi9

RUN yum install -y openssl-libs && yum clean all

COPY --from=builder /app/target/release/salvo-vite-test /usr/local/bin/

EXPOSE 8698
CMD ["salvo-vite-test"]