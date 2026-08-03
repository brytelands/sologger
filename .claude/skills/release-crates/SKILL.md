---
name: release-crates
description: Bump versions and publish the sologger workspace crates to crates.io in dependency order. Use when asked to cut a release, publish a crate, or bump versions across the workspace — the ordering matters because workspace members depend on each other through crates.io.
---

# Release the sologger crates

This workspace has an unusual property that makes releases order-dependent: **members depend on each
other by published version, not by path** — `sologger/Cargo.toml` reads
`sologger_log_context = "0.2.4"`. (A `[patch.crates-io]` section in the root `Cargo.toml` redirects
those requirements to local paths for development builds, but published packages carry only the
version requirements.)

That means a downstream crate cannot be published until its dependency's new version is already live
on crates.io, and every version bump has to be hand-propagated into the dependents' manifests.

Publishing is irreversible — a crates.io version can be yanked but never replaced or deleted.
**Never run `cargo publish` without explicit confirmation for that specific crate and version.**

## Dependency graph

```
sologger_log_context  (no internal deps)
├── sologger_idl_decoder
├── sologger_log_transformer
├── sologger_log_transformer_wasm ── also depends on sologger_idl_decoder
├── sologger_log_transport ───────── (otel and webhook features only)
└── sologger ────────── also depends on sologger_idl_decoder, sologger_log_transformer,
                        sologger_log_transport
```

**Publish order:** `sologger_log_context` → `sologger_idl_decoder` → then
`sologger_log_transformer`, `sologger_log_transformer_wasm`, `sologger_log_transport` in any
order → then `sologger` last.

`sologger` itself has never been published to crates.io (the libraries have), and neither has
`sologger_idl_decoder` (new in the Phase 2 work, locally at 0.1.0). Confirm with the user
whether this release is meant to change that.

## Step 1 — establish the current state

```bash
grep -H '^version' */Cargo.toml
for c in sologger_log_context sologger_idl_decoder sologger_log_transformer \
         sologger_log_transport sologger_log_transformer_wasm sologger; do
  echo -n "$c published: "
  curl -s "https://crates.io/api/v1/crates/$c" | python3 -c \
    "import sys,json;print(json.load(sys.stdin).get('crate',{}).get('max_version','NOT PUBLISHED'))"
done
```

Versions are **not** in lockstep — as of this writing: log_context locally 0.3.0 (published 0.2.4),
idl_decoder locally 0.1.0 (never published), log_transformer 0.2.5, log_transformer_wasm 0.2.5,
sologger 0.2.4, and log_transport locally at **0.4.0** (published: 0.2.3 — 0.3.0 carried the
OpenTelemetry 0.32 migration; 0.4.0 adds Solana trace/metric export and the webhook transport;
none of these have been published). Do not "harmonize" versions unless asked.

Ask which crates are being released and at what versions. Only bump crates whose source actually
changed; publishing an unchanged crate at a new version is noise.

## Step 2 — bump, and propagate into dependents

For each crate you bump, update **both** its own `version` and every dependent's requirement:

| Bumping | Also update in |
|---|---|
| `sologger_log_context` | `sologger-idl-decoder/Cargo.toml`, `sologger-log-transformer/Cargo.toml`, `sologger-log-transformer-wasm/Cargo.toml`, `sologger-log-transport/Cargo.toml`, `sologger/Cargo.toml` |
| `sologger_idl_decoder` | `sologger-log-transformer-wasm/Cargo.toml`, `sologger/Cargo.toml` |
| `sologger_log_transformer` | `sologger/Cargo.toml` |
| `sologger_log_transport` | `sologger/Cargo.toml` |

Missing a propagation is the classic failure here: everything builds fine against the *old* published
version and the release quietly ships nothing.

```bash
grep -rn 'sologger_log' */Cargo.toml    # review every occurrence after editing
```

## Step 3 — verify locally before publishing anything

The root `Cargo.toml` already carries a `[patch.crates-io]` section pointing the internal
dependencies at their local directories, so the workspace tests exercise the code being released —
**provided the patch still matches**. A patch entry only applies while the local crate's version
satisfies the dependent's requirement, so after bumping, confirm nothing fell back to the registry:

```bash
cargo tree --workspace | grep -c "crates.io.*sologger" ; # expect 0
cargo tree -p sologger -i sologger_log_context           # expect a local path, not a registry copy

cargo test  --workspace --features 'enable_logstash logstash otel'
cargo clippy --all --features 'enable_logstash enable_otel'
```

Leave the patch section in place. It is not carried into published packages, and `cargo publish`
verifies the packaged crate in isolation against the real registry — which is why publish order and
index waits (Step 4) still matter.

## Step 4 — dry run, then publish one crate at a time

```bash
cargo publish -p sologger_log_context --dry-run
```

Confirm with the user, then:

```bash
cargo publish -p sologger_log_context
```

Wait for the index to update (usually under a minute) before the next crate — the dependent's publish
will fail with "no matching package named ... found" if you race it:

```bash
curl -s https://crates.io/api/v1/crates/sologger_log_context | python3 -c \
  "import sys,json;print(json.load(sys.stdin)['crate']['max_version'])"
```

Repeat down the graph. Re-confirm before each `cargo publish`.

Note each crate's `exclude` list drops `/tests`, `/benches`, `/docs`, `/.github` from the package —
that is intentional; a `--dry-run` warning about missing test files is not a problem.

## Step 5 — after publishing

- Refresh the lockfile: `cargo update -p <crate>` — `Cargo.lock` is tracked, so commit the change.
- If `sologger_log_context` or `sologger_log_transformer_wasm` changed, run the `wasm-sync` skill so
  the committed dev copy under `sologger-ui/public/` matches. The live site takes care of itself —
  the Pages workflow rebuilds WASM from source on the next push to `main`.
- Docker images embed the binary; mention whether they need rebuilding
  (`docker build -f Dockerfile-logstash --tag sologger-logstash .`).
- Tag the release in git only if the user asks.

## Report

List each crate published with its version, anything you bumped but deliberately did not publish,
and what downstream artifacts (the committed WASM dev copy, Docker images) are now out of date.
