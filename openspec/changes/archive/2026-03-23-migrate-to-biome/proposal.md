## Why

ESLint + Prettier is heavy, slow, and requires multiple config files and many packages. Biome replaces both with a single fast tool and one config file.

## What Changes

- Remove `eslint`, `@eslint/js`, `eslint-plugin-vue`, `globals`, `@vue/eslint-config-typescript`, `vue-eslint-parser`, `@vue/eslint-config-prettier`, `prettier` from devDependencies
- Remove `eslint.config.js` and `.prettierrc.json`
- Install `@biomejs/biome` as devDependency
- Add `biome.json` config (singleQuote, lineWidth: 100, experimental Vue support enabled)
- Update `lint` and `format` scripts in `package.json`

## Capabilities

### New Capabilities

- `biome-tooling`: Single tool replacing ESLint + Prettier for formatting and linting `.ts`, `.js`, and `.vue` files (experimental Vue support via `html.experimentalFullSupportEnabled: true`)

### Modified Capabilities

## Impact

- `package.json`: devDependencies and scripts changed
- `eslint.config.js`: deleted
- `.prettierrc.json`: deleted
- `biome.json`: new file added at project root
- Vue template linting (eslint-plugin-vue rules) is intentionally dropped — accepted trade-off
- `vite-plugin-checker` is kept for TypeScript type-checking during dev
