---
name: wasm-sync
description: Rebuild the sologger-log-transformer-wasm package and sync it into sologger-ui/public for local dev and CI. Use whenever Rust parser changes need to reach the browser app, when the committed WASM bundle looks stale, or after touching sologger-log-context / sologger-log-transformer-wasm.
---

# Sync the WASM parser into the UI

For local dev, the browser app loads a **committed build artifact**, not a live dependency. Rust
changes are invisible to `npm run dev` (and to the `ui.yml` CI build) until this sync runs.

```
sologger-log-transformer-wasm/pkg/                          wasm-pack output (gitignored)
  -> sologger-ui/public/sologger-log-transformer-wasm/pkg/   tracked; what `npm run dev` serves
```

The **deployed** site is not affected by this copy: `.github/workflows/pages.yml` rebuilds the WASM
from source and overwrites `public/.../pkg/` during every Pages deploy. This sync is for local dev
and CI parity.

## Step 0 — confirm local resolution

`sologger-log-transformer-wasm/Cargo.toml` depends on `sologger_log_context` by published version,
redirected to the sibling directory by the `[patch.crates-io]` section in the root `Cargo.toml`. If
your change lives in `sologger-log-context/src/`, confirm the patch is resolving before building —
the output must show the local path, not a registry copy:

```bash
cargo tree -p sologger_log_transformer_wasm -i sologger_log_context
# expect: sologger_log_context vX.Y.Z (/.../sologger/sologger-log-context)
```

If a registry copy shows up instead, the patch section was removed or a version bump wasn't
propagated — fix that first (see the root `AGENTS.md`).

If the change is only in `sologger-log-transformer-wasm/src/`, skip this step.

## Step 1 — build

```bash
cd sologger-log-transformer-wasm
wasm-pack build --target web
```

`--target web` is required; the UI imports the ES module form directly. Other targets emit an
incompatible loader.

## Step 2 — copy into the UI

Copy the six runtime files. Do **not** copy `pkg/.gitignore` (it would make git ignore the
destination) and do not bother with `pkg/README.md`.

```bash
cd "$(git rev-parse --show-toplevel)"
for f in package.json \
         sologger_log_transformer_wasm.js \
         sologger_log_transformer_wasm.d.ts \
         sologger_log_transformer_wasm_bg.js \
         sologger_log_transformer_wasm_bg.wasm \
         sologger_log_transformer_wasm_bg.wasm.d.ts; do
  cp "sologger-log-transformer-wasm/pkg/$f" \
     "sologger-ui/public/sologger-log-transformer-wasm/pkg/$f"
done
```

Verify only the intended files moved:

```bash
diff -rq sologger-log-transformer-wasm/pkg sologger-ui/public/sologger-log-transformer-wasm/pkg
# expected output: only .gitignore and README.md reported as "Only in ...-wasm/pkg"
git status --short sologger-ui/public/
```

## Step 3 — check the JS API did not shift

If you added, renamed, or changed the signature of an `#[wasm_bindgen]` export, the UI callers need
updating too:

```bash
grep -rn "sologger_log_transformer_wasm" sologger-ui/src
```

`ConvertView.vue` and `HomeView.vue` are the callers. Remember slot numbers cross the boundary as
`BigInt` — a plain JS number where Rust expects `u64` throws at the bindgen layer.

## Step 4 — verify in the app

```bash
cd sologger-ui && npx vitest run   # all 12 should pass
npm run dev
```

Exercise `/convert`, which is the most direct path through the WASM module.

## Step 5 — the published bundle

Nothing extra to do: `.github/workflows/pages.yml` rebuilds the WASM from source on every deploy, so
landing the Rust change on `main` is what updates the live site. Committing the synced `public/`
copy keeps dev and CI consistent with it — see the `deploy-ui` skill for the pipeline details.

## Report

State which crates were rebuilt, whether you added a `[patch.crates-io]` section and whether you left
it in place, and whether the synced `public/` copy still needs committing.
