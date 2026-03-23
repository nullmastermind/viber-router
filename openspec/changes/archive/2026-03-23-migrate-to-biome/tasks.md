## 1. Remove ESLint and Prettier

- [x] 1.1 Uninstall ESLint and Prettier packages: `eslint`, `@eslint/js`, `eslint-plugin-vue`, `globals`, `@vue/eslint-config-typescript`, `vue-eslint-parser`, `@vue/eslint-config-prettier`, `prettier`
- [x] 1.2 Delete `eslint.config.js`
- [x] 1.3 Delete `.prettierrc.json` ← (verify: none of the removed packages remain in package.json, config files are gone)

## 2. Install and Configure Biome

- [x] 2.1 Install `@biomejs/biome` as devDependency
- [x] 2.2 Create `biome.json` with: `formatter.lineWidth: 100`, `javascript.formatter.quoteStyle: "single"`, `html.experimentalFullSupportEnabled: true`, files include `**/*.vue` ← (verify: biome.json exists, settings match spec, `npx biome --version` works)

## 3. Update Scripts

- [x] 3.1 Update `package.json` `lint` script to `biome lint ./src`
- [x] 3.2 Update `package.json` `format` script to `biome format --write ./src` ← (verify: scripts.lint and scripts.format contain biome commands, `npm run format` runs without error)
