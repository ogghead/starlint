# starlint-example

A minimal example project demonstrating how to use [starlint](https://github.com/ogghead/starlint), a fast Rust-based JavaScript/TypeScript linter.

## Setup

```bash
npm install
```

## Usage

Lint your code:

```bash
npm run lint
```

Auto-fix issues:

```bash
npm run lint:fix
```

## What's inside

- **`src/index.js`** — Clean code that passes all rules
- **`src/bad-practices.js`** — Intentional errors (`debugger`, `eval`) to demonstrate error-level diagnostics
- **`src/style-issues.js`** — Intentional warnings (`console.log`) to demonstrate warning-level diagnostics
- **`starlint.toml`** — Configuration with rules and overrides

## Configuration

`starlint.toml` configures which rules are active and their severity:

```toml
[rules]
"no-debugger" = "error"
"no-console" = "warn"
"no-eval" = "error"

[[overrides]]
files = ["**/*.test.js"]
[overrides.rules]
"no-console" = "off"
```

See the [starlint documentation](https://github.com/ogghead/starlint) for the full list of available rules and configuration options.

## License

MIT
