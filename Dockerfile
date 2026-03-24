# Stage 1: Build UI
FROM node:22-slim AS ui-builder
WORKDIR /app
RUN npm i -g bun
COPY package.json bun.lock* tsconfig.json quasar.config.ts biome.json postcss.config.js index.html ./
COPY src/ ./src/
COPY public/ ./public/
RUN bun install --frozen-lockfile
RUN bun run build

# Stage 2: Build Rust API
FROM rust:latest AS api-builder
WORKDIR /app
COPY viber-router-api/ ./viber-router-api/
WORKDIR /app/viber-router-api
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app

# Copy built binary
COPY --from=api-builder /app/viber-router-api/target/release/viber-router-api ./viber-router-api

# Copy built SPA
COPY --from=ui-builder /app/dist/spa ./dist/spa

# Copy migrations
COPY --from=api-builder /app/viber-router-api/migrations ./migrations

ENV SPA_DIR=/app/dist/spa
ENV HOST=0.0.0.0
ENV PORT=3333

EXPOSE 3333

CMD ["./viber-router-api"]
