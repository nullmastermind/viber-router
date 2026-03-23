## ADDED Requirements

### Requirement: Biome replaces ESLint and Prettier
The project SHALL use `@biomejs/biome` as the sole tool for formatting and linting. ESLint, Prettier, and all related plugins SHALL be removed from devDependencies.

#### Scenario: No ESLint or Prettier packages remain
- **WHEN** `package.json` devDependencies are inspected
- **THEN** none of `eslint`, `prettier`, `eslint-plugin-vue`, `@eslint/js`, `globals`, `@vue/eslint-config-typescript`, `vue-eslint-parser`, `@vue/eslint-config-prettier` are present
- **THEN** `@biomejs/biome` is present

#### Scenario: No ESLint or Prettier config files remain
- **WHEN** the project root is inspected
- **THEN** `eslint.config.js` does not exist
- **THEN** `.prettierrc.json` does not exist
- **THEN** `biome.json` exists

### Requirement: Biome config preserves existing style preferences
The `biome.json` SHALL configure formatting to match the previous Prettier settings: single quotes and line width of 100.

#### Scenario: Single quotes enforced
- **WHEN** `biome.json` is inspected
- **THEN** `javascript.formatter.quoteStyle` is `"single"`

#### Scenario: Line width set to 100
- **WHEN** `biome.json` is inspected
- **THEN** `formatter.lineWidth` is `100`

### Requirement: Biome experimental Vue support is enabled
The `biome.json` SHALL enable `html.parser.allowWrongLineTerminators` or `html.experimentalFullSupportEnabled` (whichever is the correct Biome v2 flag) so that `.vue` files are processed.

#### Scenario: Vue files are included in lint and format scope
- **WHEN** `biome.json` files include patterns are inspected
- **THEN** `**/*.vue` is included

### Requirement: npm scripts use Biome
The `package.json` `lint` and `format` scripts SHALL invoke Biome instead of ESLint and Prettier.

#### Scenario: lint script uses Biome
- **WHEN** `package.json` scripts.lint is inspected
- **THEN** the value contains `biome lint`

#### Scenario: format script uses Biome
- **WHEN** `package.json` scripts.format is inspected
- **THEN** the value contains `biome format --write`

### Requirement: vite-plugin-checker is retained
The `vite-plugin-checker` devDependency SHALL remain in `package.json` to continue providing TypeScript type-checking during development.

#### Scenario: vite-plugin-checker still present
- **WHEN** `package.json` devDependencies are inspected
- **THEN** `vite-plugin-checker` is present
