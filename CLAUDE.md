# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

When asked about the codebase, project structure, or to find code, always use the augment-context-engine MCP tool (codebase-retrieval) in the root workspace first before reading individual files.

## Project Overview

Viber Router is a monorepo with a Vue 3 / Quasar frontend SPA and a Rust (Axum) backend API.

## Commands

All commands use `bun` as the JS package manager and `just` as the task runner.

```bash
# Frontend
bun install              # Install dependencies
bun run dev              # Start Quasar dev server (or: just dev-ui)
bun run build            # Production build
bun run lint             # Biome lint src/
bun run format           # Biome format src/

# Backend (from viber-router-api/)
cargo run                # Start API server (or: just dev-api)
cargo check              # Type check
cargo clippy -- -D warnings  # Lint

# Full check (type-check + lint both frontend and backend)
just check
```

## Architecture

### Frontend (`src/`)
- **Framework:** Vue 3 + Quasar 2 + Vite (via `@quasar/app-vite`)
- **State:** Pinia stores in `src/stores/`
- **Routing:** Vue Router (hash mode) configured in `src/router/routes.ts`
- **HTTP:** Axios boot plugin in `src/boot/axios.ts` — exposes `$axios` and `$api` globals
- **Layout:** Single `MainLayout.vue` shell with left drawer, pages render inside `<router-view />`

### Backend (`viber-router-api/`)
- **Framework:** Axum 0.8 on Tokio
- **Database:** PostgreSQL via `sqlx` (async, compile-time checked queries)
- **Cache:** Redis via `deadpool-redis`
- **Config:** Environment variables loaded from `.env` via `dotenvy` (see `.env.example`)
- **Shared state:** `AppState { db: PgPool, redis: Pool }` passed to handlers via Axum state
- **Routes:** Defined in `src/routes/mod.rs`, handlers in sibling files (e.g., `health.rs`)

### Key Config Files
- `biome.json` — Linter/formatter. Line width 100, single quotes, recommended rules. Covers `*.vue`, `*.ts`, `*.js`.
- `quasar.config.ts` — Vite/Quasar build config. Note: still references a non-existent `eslint.config.js` in `vite-plugin-checker` (stale from ESLint→Biome migration).
- `.editorconfig` — 2-space indent, LF line endings for JS/TS/Vue files.

## Conventions

- TypeScript strict mode is enabled
- Biome handles both linting and formatting (no ESLint/Prettier)
- Rust code must pass `clippy` with `-D warnings` (warnings are errors)
- Backend env vars: `DATABASE_URL`, `REDIS_URL` (required); `HOST`, `PORT`, `RUST_LOG` (optional with defaults)
