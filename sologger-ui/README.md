# Sologger UI

A browser app for watching and analyzing Solana program logs: a live log stream per program, transaction lookup with a
compute-unit flamegraph, and raw-log conversion. Logs are parsed in the browser by sologger's Rust parser compiled to
WebAssembly.

The hosted version is at <https://brytelands.github.io/sologger/>.

## Run it locally

**With Docker** (no Node.js needed), from the repository root:

```shell
docker compose up sologger-ui
```

Then open <http://localhost:8080/sologger/>. Or without compose:

```shell
docker build -t sologger-ui:local sologger-ui
docker run --rm -p 8080:8080 sologger-ui:local
```

**With Node.js** (for development, with hot reload):

```shell
cd sologger-ui
npm install
npm run dev
```

The app runs entirely in the browser and connects to the Solana RPC you pick in it (devnet by default). It does not
talk to a running sologger service.

## Tests

```shell
npx vitest run         # unit tests
npm run test:e2e       # Playwright; first run `npx playwright install chromium`
```

See [AGENTS.md](./AGENTS.md) for how the app is put together.
