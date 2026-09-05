# Multi-stage build: frontend (Vite) -> backend (Rust, Typst engine embedded)
# -> slim runtime. The backend serves the built frontend and persists data
# under /data (mount a volume there).

# ---------- frontend ----------
FROM node:24-alpine AS web
WORKDIR /web
COPY frontend/package-lock.json frontend/package.json ./
RUN npm ci
COPY frontend ./
RUN npm run build

# ---------- backend ----------
FROM rust:1-slim AS backend
WORKDIR /build
# dependency cache layer: build with a stub first so source changes stay fast
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release --locked
COPY backend ./
RUN touch src/main.rs && cargo build --release --locked

# ---------- runtime ----------
FROM debian:stable-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend /build/target/release/drill-backend /app/drill-backend
COPY --from=web /web/dist /app/dist
RUN mkdir -p /data
ENV DRILL_HOST=0.0.0.0 \
    DRILL_PORT=8787 \
    DRILL_DIST=/app/dist \
    DRILL_DATA=/data \
    DRILL_ROOT=/data/bank
VOLUME ["/data"]
EXPOSE 8787
ENTRYPOINT ["/app/drill-backend"]
