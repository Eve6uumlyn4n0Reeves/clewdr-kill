# syntax=docker/dockerfile:1

# ============== Frontend build ==============
FROM node:20-alpine AS frontend-builder
WORKDIR /app/frontend

# 复制package文件以利用Docker层缓存
COPY frontend/package.json frontend/package-lock.json* ./

# 安装所有依赖（包括开发依赖，因为需要tsc和vite进行构建）
RUN npm ci && npm cache clean --force

# 复制源代码
COPY frontend .

# 构建前端，启用生产优化
RUN npm run build

# ============== Backend build ==============
FROM rustlang/rust:nightly-slim AS backend-builder
WORKDIR /app

# 设置构建优化环境变量
ENV CARGO_TERM_COLOR=always
ENV CARGO_NET_RETRY=10
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV RUSTFLAGS="-C target-cpu=native"

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    cmake \
    make \
    g++ \
    perl \
    golang \
    && rm -rf /var/lib/apt/lists/*

# 复制Cargo文件进行依赖预构建
COPY Cargo.toml Cargo.lock ./

# 创建虚拟main.rs进行依赖构建
RUN mkdir -p backend && \
    echo "fn main() {}" > backend/main.rs && \
    echo "pub fn lib() {}" > backend/lib.rs

# 预构建依赖（利用Docker层缓存）
RUN cargo build --release --jobs $(nproc)

# 复制实际源代码
COPY backend ./backend

# 触发重新编译并构建最终二进制
RUN touch backend/main.rs backend/lib.rs && \
    cargo build --release --jobs $(nproc)

# ============== Runtime ==============
FROM debian:bookworm-slim
WORKDIR /app

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libsqlite3-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# 创建非特权用户
RUN groupadd -g 1001 clewdr && \
    useradd -u 1001 -g clewdr -s /bin/sh -m clewdr

# 复制二进制文件
COPY --from=backend-builder /app/target/release/clewdr-ban-tool /app/clewdr
RUN chmod +x /app/clewdr

# 复制前端静态资源
COPY --from=frontend-builder /app/static /app/static

# 创建必要目录并设置权限
RUN mkdir -p /app/ban_prompts /app/data && \
    chown -R clewdr:clewdr /app

# 切换到非特权用户
USER clewdr

# 配置卷挂载
VOLUME ["/app/ban_prompts", "/app/data"]

# 暴露端口
EXPOSE 8484

# 设置环境变量优化性能
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# 健康检查 - 使用更轻量的检查
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8484/api/health || exit 1

# 启动应用
CMD ["/app/clewdr"]
