# Viber Router

Viber Router is an Anthropic API relay proxy and load balancer with a web admin UI. You put it in front of your Anthropic-compatible API keys, point your clients at it instead of `api.anthropic.com`, and it handles routing, failover, and monitoring for you. Clients keep using the Anthropic SDK as-is — only the base URL and key change.

## What it does

A request comes in on `/v1/*` carrying a proxy-issued key. Viber Router reads that key to figure out which group it belongs to, picks the first healthy server in the group, rewrites the model name if that server expects a different alias, and forwards the request upstream. If the upstream answers with a status code you've flagged for failover (429, 500, 502, and 503 by default), it moves on to the next server in the group and tries again. Streaming responses are passed straight through, and usage and timing data are parsed out of the stream as it flows.

Around that core relay path sit the things you'd want from a proxy you actually run in production:

- **Group-based routing** — each group is an ordered list of upstream servers, each with its own key and its own model-name mapping.
- **Per-server model mapping** — send `claude-opus-4-5` and have one server receive that name while another receives whatever alias it expects.
- **TTFT monitoring** — Time-to-First-Token is recorded for every streaming request, with average, p50, and p95 views, a scatter chart, and a navigable time window.
- **Token and cost tracking** — token usage is parsed inline and priced against the model table.
- **Proxy logs** — a searchable request log showing status, latency, server, and group.
- **Telegram alerts** — a notification fires when an upstream returns a monitored status, with a configurable code list and cooldown so you're not spammed.
- **Dynamic per-server keys** — a proxy key can embed override keys for individual servers when you need them.

Operationally it stays out of your way. SQLx migrations run on startup, so there's no manual schema step. The high-volume log tables (`proxy_logs`, `ttft_logs`, token usage, and uptime checks) are partitioned by month and old partitions are dropped automatically once they pass the retention window. Admin access is gated by a single bearer token. And the whole thing — the Rust API binary plus the compiled Vue SPA — ships as one Docker image.

## How it's built

The frontend is a Vue 3 / Quasar 2 single-page app built with Vite, using Pinia for state and Axios for HTTP. The backend is Rust on Axum 0.8 and Tokio. PostgreSQL (via SQLx) is the system of record, and Redis (via deadpool-redis) handles caching and rate limiting. In production both halves are served from the same container.

## Configuration

The backend reads its configuration from a `.env` file, or directly from the environment. Three values are required:

- `DATABASE_URL` — PostgreSQL connection string, e.g. `postgres://user:pass@localhost:5432/viber_router`
- `REDIS_URL` — Redis connection string, e.g. `redis://localhost:6379`
- `ADMIN_TOKEN` — the bearer token that protects the admin UI and API

The rest have sensible defaults: `HOST` (`0.0.0.0`), `PORT` (`3333`), `DATABASE_MAX_CONNECTIONS` (`50`), `REDIS_MAX_CONNECTIONS` (`30`), `RUST_LOG` (`info`), and `LOG_RETENTION_DAYS` (`30`, the number of days log partitions are kept before being dropped).

For local development, copy `viber-router-api/.env.example` to `viber-router-api/.env` and fill in the required values.

## Deployment

The simplest way to run Viber Router is Docker Compose alongside Postgres and Redis. Drop this into a `docker-compose.yml`:

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

Then bring it up with `docker compose up -d`. The admin UI will be at `http://localhost:3333`; log in with the `ADMIN_TOKEN` you set. Because `pull_policy: always` is in place, a later `docker compose pull && docker compose up -d` will fetch and roll out the latest image.

If you'd rather not use Compose and already have Postgres and Redis elsewhere, a single container works too:

```bash
docker run -d \
  -p 3333:3333 \
  -e DATABASE_URL="postgres://user:pass@host:5432/viber_router" \
  -e REDIS_URL="redis://host:6379" \
  -e ADMIN_TOKEN="your-secret-token" \
  --name viber-router \
  nullmastermind/viber-router:latest
```

## Development

You'll need stable Rust, Node.js 22+, [Bun](https://bun.sh/), the [just](https://github.com/casey/just) task runner, and a local PostgreSQL and Redis. Note that the backend uses SQLx compile-time query checking, so `cargo check` needs a reachable database.

Get set up by installing the frontend dependencies and creating your backend config:

```bash
bun install
cp viber-router-api/.env.example viber-router-api/.env
# then edit viber-router-api/.env to set DATABASE_URL, REDIS_URL, ADMIN_TOKEN
```

Run the two halves in separate terminals. `just dev-ui` starts the Quasar HMR server on `http://localhost:9000` and `just dev-api` starts the Rust API on `http://localhost:3333`; the dev server proxies API calls to the backend automatically, so you just open port 9000.

Before committing, run `just check`. It runs `vue-tsc`, `biome lint`, `cargo check`, and `cargo clippy -D warnings` across both halves — and since clippy runs with `-D warnings`, anything less than clean fails.

To build the image yourself, `just docker-build` produces `nullmastermind/viber-router:latest` locally and `just docker-push` builds and pushes it to Docker Hub.
