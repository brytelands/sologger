# AGENTS.md — sologger-ui

Vue 3 + Vite single-page app for viewing and converting Solana logs in the browser. Read the root
`AGENTS.md` first for repo-wide rules; this file covers the UI only.

## Stack

- **Vue 3** — views use the Options API (`export default { data() … }`); `src/composables/` uses the
  Composition API. Both styles are current; match whichever file you are in.
- **Vite 6**, `base: '/sologger/'` (GitHub Pages). Router uses `createWebHistory('/sologger/')` — the
  two must stay in sync.
- **PrimeVue 4** mounted with `theme: "none"`. There is no PrimeVue theme; styling comes from
  Tailwind plus the hand-maintained per-component CSS in `src/assets/primevue/`. The only
  PrimeVue component in use is Toast, so `src/assets/primevue/tailwind.css` imports just
  `common` + `toast`; add an `@import` there when you start using another component
  (importing all ~84 used to make the stylesheet ~677 kB instead of ~59 kB).
- **Tailwind 3** + PostCSS, with `tailwindcss-primeui`.
- **SlickGrid** for the log table, **Chart.js** (`vue-chartjs`) for stats; the CU flamegraph is
  hand-rolled SVG (`CuFlamegraph.vue`).
- **@coral-xyz/anchor** only for fetching on-chain IDLs (`Program.fetchIdl`). IDL event decoding is
  done by the Rust decoder (`sologger-idl-decoder`) through the WASM module — the BorshCoder path
  was retired in Phase 6.3.
- **Vitest** for tests, `environment: 'node'`, only matching `src/tests/**/*.test.js`. The suite
  loads the committed WASM build via `initSync` and exercises the real decoder.

The last Handsontable leftovers (an `optimizeDeps` entry and the `vite-plugin-wasm-pack`
devDependency) were removed in the 2026-08 cleanup; if you see Handsontable references anywhere,
they are stale.

## Layout

```
src/main.js            App bootstrap; applies the stored theme to <html data-theme> before mount.
src/router/index.js    Four routes: / (home), /convert, /lookup, /about. Each view is lazy-loaded
                       as its own chunk; router.onError turns a failed chunk load (a tab
                       opened before a redeploy) into a full load of the target URL.
src/views/
  HomeView.vue         ~900 lines. Live log stream (sockets live in useLogStream), filters,
                       IDL decode, exports. The stream e2e tests stub the RPC socket with
                       mockLogStream in e2e/ui.spec.js.
  ConvertView.vue      Paste raw logs -> structured output via the WASM module.
  LookupView.vue       Paste a signature -> getTransaction from a configurable RPC -> WASM parse
                       -> summary cards + CU flamegraph + LogsTable. Deep-linkable via ?sig=.
                       Traces the transaction's programs to their published on-chain IDLs
                       (Program.fetchIdl, capped candidate list from idlCandidatePrograms) and
                       re-parses with them, so events/error names decode with zero setup;
                       opt-out checkbox, manual IDL upload still available.
  AboutView.vue
src/components/
  LogsTable.vue        ~690 lines. SlickGrid table, CPI depth rendering, mobile card view.
                       Its <style> is unscoped on purpose: SlickGrid injects cell HTML via
                       innerHTML, which scoped styles cannot reach — and everything a grid
                       formatter interpolates must go through its escapeHtml helper (log
                       content is attacker-controlled).
  StatsGrid.vue        Chart.js panels (CU consumption, log level distribution) plus the
                       latest-transaction CU flamegraph.
  CuFlamegraph.vue     SVG icicle of one transaction's CPI tree: depth = row, width = consumed
                       CU, hue = program (fixed first-appearance order, "Other" past eight,
                       validated palette for both themes). Layout math in useCuFlamegraph.js.
  ProgramIdForm.vue    Program ID entry, plus a "Popular programs" picker (POPULAR_PROGRAMS:
                       six well-known programs with steady devnet traffic, same IDs on
                       mainnet) that fills the field; the user still clicks Add Program.
  ProgramList.vue      Monitored-program chips: status dot + label + remove, fed by
                       HomeView's programStatuses computed ({id, status} objects).
  CollapsibleSection.vue  Card/bare section with a persisted open state
                          (localStorage sologger_section_<storageKey>).
src/composables/useLogStream.js     createLogStream — HomeView's live logsSubscribe sockets (one per
                                    program): de-duplicates transactions by signature (several
                                    monitored programs report the same one), reconnects dropped
                                    sockets with backoff, per-program reactive status. Takes the
                                    WebSocket class as an option, so logStream.test.js drives it
                                    with a fake socket and fake timers.
src/composables/useTheme.js         Theme state, persisted to localStorage under key 'theme'.
src/composables/useIdlDecoder.js    decodeWithIdl(idl, log) — instruction matching + event decoding
                                    via the WASM decode_program_data export, shared with tests.
src/composables/useLogMapper.js     mapLogContext — parsed LogContext -> LogsTable row shape,
                                    shared by HomeView and LookupView.
src/composables/useCuFlamegraph.js  Flamegraph layout math (tree rebuild, CU-proportional spans,
                                    palette slot assignment), shared with tests.
src/composables/useLogSanitizer.js  sanitizeLogMessage — whitespace normalization, shared by
                                    HomeView and ConvertView.
src/tests/             Vitest specs.
public/sologger-log-transformer-wasm/pkg/   Committed WASM build (see below).
```

`HomeView.vue` and `LogsTable.vue` carry most of the logic. Prefer extending them over adding new
top-level views unless the feature really is a new page.

## The WASM module

The parser is Rust compiled to WebAssembly. The app loads it from `public/`, *not* from node_modules:

```js
import init, { … } from '../../public/sologger-log-transformer-wasm/pkg/sologger_log_transformer_wasm.js'
```

The contents of that `pkg/` directory are **committed build output**, copied by hand from
`sologger-log-transformer-wasm/pkg/` after a `wasm-pack build --target web`. Rust-side changes are
invisible to the UI until that copy happens — use the `wasm-sync` skill.

Slot numbers cross the boundary as `BigInt`. Passing a plain JS number where the Rust side expects
`u64` throws at the bindgen layer.

## Commands

```bash
npm install
npm run dev        # vite dev server
npm run build      # -> dist/
npm run preview    # builds, then serves
npx vitest run     # there is no `npm test` script
npm run test:e2e   # Playwright; first run `npx playwright install chromium`
docker compose up sologger-ui   # from the repo root: Dockerfile + nginx.conf here, :8080/sologger/
```

The Docker image serves the build under `/sologger/` because of the base path; if you change
`base`, change the `COPY` destination in `Dockerfile` and the locations in `nginx.conf` too.

Publishing to GitHub Pages is automatic: `.github/workflows/pages.yml` builds fresh WASM + the UI
and deploys `dist/` on every push to `main` that touches this directory — see the `deploy-ui` skill.

## Testing

All Vitest specs pass on a clean checkout, and CI runs them along with the Playwright suite
(`.github/workflows/ui.yml`: `npm ci`, `npx vitest run`, `npm run build`, `npm run test:e2e`).

`e2e/ui.spec.js` drives the real views in Chromium against the Vite dev server (started by
`playwright.config.js` on port 5199). All off-origin traffic is stubbed, so it needs no live RPC; the
`getTransaction` stub rejects requests lacking `maxSupportedTransactionVersion >= 1`, as a node does.
It loads the committed WASM build, so parser changes need `wasm-sync` before they show up here
(`pages.yml` runs it against freshly built WASM before deploying). Set
`PLAYWRIGHT_CHROMIUM_PATH` to use an already-installed Chromium instead of downloading one.

The suite initializes the committed WASM build with `initSync` (reading the `.wasm` bytes from
`public/` via `fs`), so `decodeWithIdl` and `add_idl` enrichment are tested against the real Rust
decoder — including a captured mainnet Raydium SwapEvent payload. Shared logic lives in
framework-free composables (`useIdlDecoder`, `useLogMapper`, `useCuFlamegraph`) imported by both
the views and `src/tests/` — change it there and the tests cover it. Keep composables free of Vue
imports so they stay testable in the node environment.

## Conventions

- Two-space indent, `export default` for components.
- Tailwind utilities in templates first; reach for `src/style.css` or `src/assets/tailwind.css` only
  for genuinely global rules.
- Charts must read colors from the active theme rather than relying on CSS `filter: invert(1)` — that
  approach was deliberately removed.
- User-visible state that should survive a reload (program IDs, RPC URLs, explorer preference, theme)
  goes to `localStorage`; view configuration that should be shareable goes to URL query params.
  `HomeView.vue` already does both — follow its pattern.
- Notify via the PrimeVue `Toast` service (registered in `main.js`), not `alert()`.
