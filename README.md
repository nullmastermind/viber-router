# Viber Router

A high-performance Anthropic API relay proxy and load balancer with a web-based admin UI.

Drop Viber Router in front of your Anthropic API keys to get automatic failover, per-group server routing, TTFT monitoring, and Telegram alerts — all manageable through a Vue 3 dashboard.

---

## Features

- **Drop-in Anthropic proxy** — forwards `/v1/*` requests transparently; clients need only swap the base URL and API key
- **Group-based routing** — each group holds an ordered list of upstream servers with independent API keys and failover rules
- **Automatic failover** — retries the next server in the group on configurable HTTP status codes (default: 429, 500, 502, 503)
- **Model name mapping** — rewrite model names per upstream server (e.g. send `claude-opus-4-5` to a server that expects a different alias)
- **`count-tokens` routing** — route token-counting requests to a dedicated default server
- **TTFT monitoring** — records Time-to-First-Token for every streaming request; view avg / p50 / p95 stats, scatter chart, and navigate custom time windows
- **Proxy logs** — searchable request log with status, latency, server, and group columns
- **Telegram alerts** — sends a notification when an upstream returns a monitored status code; configurable per-code list and cooldown window
- **Token-based admin auth** — single `ADMIN_TOKEN` protects all admin API endpoints
- **Auto migration** — SQLx migrations run automatically on startup; no manual schema management
- **Partitioned log tables** — monthly PostgreSQL partitions for `proxy_logs` and `ttft_logs` with configurable retention (default: 30 days)
- **Single Docker image** — Rust API binary and compiled Vue SPA served from one container

---

## Architecture

| Layer | Technology |
|---|---|
| Frontend | Vue 3 + Quasar 2 + Vite (SPA, hash routing) |
| Backend | Rust, Axum 0.8, Tokio |
| Database | PostgreSQL (SQLx, connection pool) |
| Cache / rate-limit | Redis (deadpool-redis) |
| State management | Pinia |
| HTTP client (frontend) | Axios |

---

## Environment Variables

The backend reads configuration from a `.env` file (or from the environment directly).

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string, e.g. `postgres://user:pass@localhost:5432/viber_router` |
| `REDIS_URL` | ✅ | — | Redis connection string, e.g. `redis://localhost:6379` |
| `ADMIN_TOKEN` | ✅ | — | Bearer token used to authenticate admin UI / API calls |
| `HOST` | | `0.0.0.0` | Address the server binds to |
| `PORT` | | `3333` | TCP port the server listens on |
| `DATABASE_MAX_CONNECTIONS` | | `50` | Maximum PostgreSQL connection pool size |
| `REDIS_MAX_CONNECTIONS` | | `30` | Maximum Redis connection pool size |
| `RUST_LOG` | | `info` | Log level filter (e.g. `debug`, `info`, `warn`) |
| `LOG_RETENTION_DAYS` | | `30` | Days to retain proxy and TTFT log partitions before dropping |

Copy `viber-router-api/.env.example` to `viber-router-api/.env` and fill in the required values for local development.

---

## Deployment

### Docker Compose (recommended)

Create a `docker-compose.yml`:

```yaml
services:
  app:
    image: nullmastermind/viber-router:latest
    pull_policy: always
    ports:
      - "3333:3333"
    environment:
      DATABASE_URL: postgres://viber:secret@db:5432/viber_router
      REDIS_URL: redis://redis:6379
      ADMIN_TOKEN: your-secret-token
    depends_on:
      db:
        condition: service_healthy
      redis:
        condition: service_started
    restart: unless-stopped

  db:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: viber
      POSTGRES_PASSWORD: secret
      POSTGRES_DB: viber_router
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U viber -d viber_router"]
      interval: 5s
      timeout: 5s
      retries: 10
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    volumes:
      - redis_data:/data
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

Then start the stack:

```bash
docker compose up -d
```

The admin UI is available at `http://localhost:3333`. Log in with the `ADMIN_TOKEN` value you set.

### Pull and update

`pull_policy: always` ensures `docker compose up -d` fetches the latest image automatically. To update a running stack:

```bash
docker compose pull
docker compose up -d
```

### Single container (no compose)

```bash
docker run -d \
  -p 3333:3333 \
  -e DATABASE_URL="postgres://user:pass@host:5432/viber_router" \
  -e REDIS_URL="redis://host:6379" \
  -e ADMIN_TOKEN="your-secret-token" \
  --name viber-router \
  nullmastermind/viber-router:latest
```

---

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- Node.js 22+
- [Bun](https://bun.sh/)
- [just](https://github.com/casey/just) — task runner
- PostgreSQL and Redis running locally

### Setup

```bash
# Install frontend dependencies
bun install

# Copy and fill in backend config
cp viber-router-api/.env.example viber-router-api/.env
# Edit viber-router-api/.env — set DATABASE_URL, REDIS_URL, ADMIN_TOKEN
```

### Run

```bash
just dev-ui    # Frontend HMR dev server (Quasar) — http://localhost:9000
just dev-api   # Rust API server — http://localhost:3333
```

During development the frontend proxies API calls to the backend automatically via Quasar's dev server configuration.

### Check (type-check + lint)

Run this before every commit:

```bash
just check
```

This runs `vue-tsc`, `biome lint`, `cargo check`, and `cargo clippy -D warnings` across both the frontend and backend.

### Build Docker image locally

```bash
just docker-build   # builds nullmastermind/viber-router:latest
just docker-push    # builds then pushes to Docker Hub
```
