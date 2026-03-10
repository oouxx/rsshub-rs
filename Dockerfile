# syntax=docker/dockerfile:1

# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.83-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ── 缓存依赖层：只复制 Cargo.toml / Cargo.lock，先把依赖编译好 ────────────────
# 这样只要依赖不变，后续改业务代码不会重新下载编译依赖
COPY Cargo.toml Cargo.lock ./

# 单 crate 直接创建占位 main.rs 即可，无需考虑 workspace
RUN mkdir src && echo 'fn main() {}' > src/main.rs

# 编译依赖（产物会被 Docker 缓存）
RUN cargo build --release

# 删除占位文件和对应编译产物
# 必须删干净，否则增量编译会跳过重新编译业务代码
RUN rm -f src/main.rs \
          target/release/rsshub-rs \
          target/release/deps/rsshub_rs*

# ── 复制真实源码并编译 ────────────────────────────────────────────────────────
COPY src    ./src

RUN cargo build --release

# ── Stage 2: 最小运行镜像 ─────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rsshub-rs .
COPY --from=builder /app/static                   ./static

EXPOSE 3000

ENV RUST_LOG=rsshub_rs=info,tower_http=info

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./rsshub-rs"]
