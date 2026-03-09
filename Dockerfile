# syntax=docker/dockerfile:1

# ── Stage 1: Build ────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# ── Step 1: 只复制所有 Cargo.toml / Cargo.lock，先缓存依赖层 ──────────────────
COPY Cargo.toml Cargo.lock ./
COPY crates/core/Cargo.toml   crates/core/Cargo.toml
COPY crates/server/Cargo.toml crates/server/Cargo.toml

# 给每个 workspace 成员创建最小占位文件
# core 是 lib crate → lib.rs
# server 是 bin crate → main.rs，内容必须能通过编译
RUN mkdir -p crates/core/src crates/server/src \
    && echo 'pub fn build_router() -> axum::Router { axum::Router::new() }' \
       > crates/core/src/lib.rs \
    && echo 'fn main() {}' \
       > crates/server/src/main.rs

# 编译依赖（这一层会被 Docker 缓存，只要 Cargo.toml 不变就不会重跑）
RUN cargo build --release -p rss-forge-server

# ── Step 2: 删除占位产物，换上真实源码 ────────────────────────────────────────
# 必须同时删除：
#   1. 占位的 src 文件
#   2. 对应的编译产物（.d 依赖描述文件 + rlib / 二进制）
#   否则增量编译时 rustc 会因为找不到旧占位文件路径而报错
RUN rm -rf crates/core/src crates/server/src \
    && rm -f target/release/rsshub-rs \
             target/release/deps/rsshub_rs* \
             target/release/deps/rss_forge_core* \
             target/release/.fingerprint/rsshub-rs-*/* \
             target/release/.fingerprint/rss-forge-core-*/*

COPY crates/core/src   crates/core/src
COPY crates/server/src crates/server/src
COPY static            static

# 真实编译（依赖层已缓存，只重编业务代码，速度快）
RUN cargo build --release -p rss-forge-server

# ── Stage 2: 最小运行镜像 ─────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rsshub-rs .
COPY --from=builder /app/static ./static

EXPOSE 3000

ENV RUST_LOG=rss_forge=info

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./rsshub-rs"]
