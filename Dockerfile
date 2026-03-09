# syntax=docker/dockerfile:1

# ── 编译阶段 ──────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

# 先只复制依赖文件，利用 Docker 缓存层
COPY Cargo.toml Cargo.lock ./

# 创建空的 main.rs 让依赖先编译（缓存 trick）
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# 再复制真正的源码
COPY src ./src
COPY static ./static

# 触发增量编译
RUN touch src/main.rs && cargo build --release

# ── 运行阶段（最小镜像）─────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/rsshub-rs .
COPY --from=builder /app/static ./static

EXPOSE 3000
CMD ["./rsshub-rs"]
