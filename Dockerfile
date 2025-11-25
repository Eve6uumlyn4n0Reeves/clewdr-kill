# syntax=docker/dockerfile:1

# ============== Frontend build ==============
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm install
COPY frontend .
RUN npm run build

# ============== Backend build ==============
FROM rust:1.81-slim-bookworm AS backend-builder
WORKDIR /app
ENV CARGO_TERM_COLOR=always
RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY frontend ./frontend
RUN cargo build --release

# ============== Runtime ==============
FROM debian:bookworm-slim
WORKDIR /app
ENV RUST_LOG=info
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates openssl && \
    rm -rf /var/lib/apt/lists/*

# 后端二进制
COPY --from=backend-builder /app/target/release/clewdr /app/clewdr

# 前端静态资源（axum external-resource 特性会从 /app/static 提供）
COPY --from=frontend-builder /app/frontend/dist /app/static

# 配置与数据目录
VOLUME ["/app/ban_prompts", "/app/data"]

EXPOSE 8484

CMD ["/app/clewdr"]
