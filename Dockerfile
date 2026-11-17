# syntax=docker/dockerfile:1.7

FROM rust:1.100.0-bookworm@sha256:ade39a710ef85f3506cffcfec3e8a24ba090badef1dff2fb798f71d878833fea AS builder
WORKDIR /workspace

COPY . .

RUN cargo build --locked --release -p gvm-mock-server \
    && strip target/release/gvm-mock-server

FROM gcr.io/distroless/cc-debian12:nonroot@sha256:9dac0a79194e45a7da0158a9c6da57b217585af0786db3845d1f0ec1a0dd182f

COPY --from=builder /workspace/target/release/gvm-mock-server /usr/local/bin/gvm-mock-server

ENTRYPOINT ["/usr/local/bin/gvm-mock-server"]
