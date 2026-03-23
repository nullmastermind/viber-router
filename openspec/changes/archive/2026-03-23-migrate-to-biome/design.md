## Context

The project currently uses ESLint (with eslint-plugin-vue, @vue/eslint-config-typescript, vue-eslint-parser, @eslint/js, globals) and Prettier (@vue/eslint-config-prettier) — 8 devDependencies, 2 config files, 2 scripts. Biome replaces all of this with one package and one config file.

## Goals / Non-Goals

**Goals:**
- Replace ESLint + Prettier with Biome for formatting and linting
- Preserve existing style preferences: single quotes, line width 100
- Enable experimental Vue SFC support in Biome
- Keep `vite-plugin-checker` for TypeScript type-checking during dev

**Non-Goals:**
- Vue template directive linting (v-if, v-for, v-model rules) — intentionally dropped
- Cross-block analysis between `<script>` and `<template>` — not supported by Biome yet
- Migrating existing lint violations — format/lint runs after migration, not before

## Decisions

**D1: Use Biome v2.x (latest) with `html.experimentalFullSupportEnabled: true`**
- Enables formatting and linting of `.vue` files including `<script>`, `<style>`, and `<template>` blocks
- Without this flag, `.vue` files are only partially processed
- Trade-off: experimental flag means occasional false positives; accepted

**D2: Drop eslint-plugin-vue entirely**
- Biome does not understand Vue template directives as first-class syntax
- Keeping eslint-plugin-vue alongside Biome would defeat the purpose of simplification
- Accepted: template-specific lint rules (v-for key, v-if/v-for conflicts) are dropped

**D3: Map Prettier config to Biome equivalents**
- `singleQuote: true` → `"quoteStyle": "single"` in `javascript.formatter`
- `printWidth: 100` → `"lineWidth": 100` in `formatter`

**D4: Scripts**
- `lint`: `biome lint ./src`
- `format`: `biome format --write ./src`
- `check` (new, optional): `biome check --write ./src` — runs lint + format together

## Risks / Trade-offs

- [Vue template lint rules dropped] → Accepted trade-off; TypeScript type-checking via `vite-plugin-checker` still catches type errors
- [Biome Vue support is experimental] → False positives possible; can suppress per-file with `// biome-ignore` comments
- [Biome v2.x is a major version] → API may differ from v1.x; use `@biomejs/biome` latest
