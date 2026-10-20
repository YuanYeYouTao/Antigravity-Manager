# --- Frontend Build Stage ---
FROM node:20-slim AS frontend-builder
ARG USE_MIRROR=auto
WORKDIR /app

# Use npm mirror if needed
RUN if [ "$USE_MIRROR" = "true" ] || ( [ "$USE_MIRROR" = "auto" ] && ! timeout 3 bash -c "</dev/tcp/www.google.com/80" 2>/dev/null ); then \
    echo "Using npm mirror..."; \
    npm config set registry https://registry.npmmirror.com; \
    else \
    echo "Using default npm registry..."; \
    fi

COPY package*.json ./
RUN --mount=type=cache,target=/root/.npm \
    npm ci --legacy-peer-deps --prefer-offline --no-audit --no-fund
COPY tsconfig.json tsconfig.node.json vite.config.ts tailwind.config.js postcss.config.cjs index.html ./
COPY public ./public
COPY src ./src
COPY src-tauri/icons ./src-tauri/icons
RUN npm run build

# --- Backend Build Stage ---
FROM rust:1-slim-bookworm AS backend-builder
ARG USE_MIRROR=auto

# Conditionally use Aliyun mirror for APT
RUN if [ "$USE_MIRROR" = "true" ] || ( [ "$USE_MIRROR" = "auto" ] && ! timeout 3 bash -c "</dev/tcp/www.google.com/80" 2>/dev/null ); then \
    echo "Using Aliyun mirror for APT..."; \
    sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources || \
    sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list; \
    else \
    echo "Using default HTTPS APT sources..."; sed -i 's#http://deb.debian.org#https://deb.debian.org#g' /etc/apt/sources.list.d/debian.sources; \
    fi

# Install build dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked apt-get -o Acquire::Retries=3 update && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
    pkg-config \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libsoup-3.0-dev \
    libjavascriptcoregtk-4.1-dev \
    perl \
    cmake \
    golang-go \
    clang \
    libclang-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Use Aliyun mirror for Cargo if needed (Sparse Index)
ENV CARGO_HTTP_MULTIPLEXING=false
RUN if [ "$USE_MIRROR" = "true" ] || ( [ "$USE_MIRROR" = "auto" ] && ! timeout 3 bash -c "</dev/tcp/www.google.com/80" 2>/dev/null ); then \
    echo "Using Aliyun mirror for Cargo..."; \
    mkdir -p "$CARGO_HOME" && \
    echo "[source.crates-io]\nreplace-with = 'aliyun'\n\n[source.aliyun]\nregistry = \"sparse+https://mirrors.aliyun.com/crates.io-index/\"" > "$CARGO_HOME/config.toml"; \
    else \
    echo "Using default Cargo registry..."; \
    fi

# Verify Go installation for BoringSSL build
RUN which go && go version || (echo "ERROR: Go compiler not found" && exit 1)

WORKDIR /app
# Copy only backend sources to keep frontend cache intact on backend-only changes
COPY src-tauri ./src-tauri

# [FIX] Copy locales for Rust compilation (needed by i18n.rs include_str!)
COPY src/locales ./src/locales

# Copy frontend dist from builder so Tauri can embed it
COPY --from=frontend-builder /app/dist ./dist

# Build the Rust backend in release mode (with BuildKit cache)
WORKDIR /app/src-tauri
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/src-tauri/target \
    cargo build --release --locked --jobs 2 --bin antigravity-tools && \
    cp target/release/antigravity-tools /tmp/antigravity-tools

# --- Final Runtime Stage ---
FROM debian:bookworm-slim
COPY --from=rust:1-slim-bookworm /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
ARG USE_MIRROR=auto
ARG SOURCE_REVISION
LABEL org.opencontainers.image.source="https://github.com/YuanYeYouTao/Antigravity-Manager" \
      org.opencontainers.image.revision=$SOURCE_REVISION \
      org.opencontainers.image.version="4.9.9-beta.2" \
      org.opencontainers.image.base.revision="54fd90b036a50a7a8772cb9b7d79a74e2c62a149"
WORKDIR /app

# Conditionally use Aliyun mirror for APT
RUN if [ "$USE_MIRROR" = "true" ] || ( [ "$USE_MIRROR" = "auto" ] && ! timeout 3 bash -c "</dev/tcp/www.google.com/80" 2>/dev/null ); then \
    echo "Using Aliyun mirror for APT..."; \
    sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list.d/debian.sources || \
    sed -i 's/deb.debian.org/mirrors.aliyun.com/g' /etc/apt/sources.list; \
    else \
    echo "Using default HTTPS APT sources..."; sed -i 's#http://deb.debian.org#https://deb.debian.org#g' /etc/apt/sources.list.d/debian.sources; \
    fi

# Install runtime dependencies
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked apt-get -o Acquire::Retries=3 update && apt-get -o Acquire::Retries=3 install -y --no-install-recommends \
    curl \
    libssl3 \
    libsqlite3-0 \
    ca-certificates \
    libgtk-3-0 \
    libwebkit2gtk-4.1-0 \
    libayatana-appindicator3-1 \
    librsvg2-2 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=backend-builder /tmp/antigravity-tools /app/antigravity-tools

# Copy frontend dist from builder
COPY --from=frontend-builder /app/dist /app/dist

# Set environment variables
ENV ABV_DIST_PATH=/app/dist
ENV RUST_LOG=info
ENV PORT=8045

# Expose the proxy/admin port
EXPOSE 8045

# Healthcheck for container orchestration
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://127.0.0.1:8045/health || exit 1

# Run the application in headless mode
ENTRYPOINT ["/app/antigravity-tools", "--headless"]
