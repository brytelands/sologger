---
name: release-crates
description: Cut a sologger release — crates.io publishes, the v<version> GitHub Release with Linux binaries, and Docker images — via the release-plz automation, or troubleshoot one. Use when asked to cut a release, publish a crate, bump versions, or check why a release did not go out.
---

# Release the sologger crates

Releases are automated with [release-plz](https://release-plz.dev) (`release-plz.toml`,
`.github/workflows/release-plz.yml`). Do not run `cargo publish` by hand — publishing is
irreversible (a version can be yanked, never replaced), and the workflow already orders, tags and
packages everything. The one exception is the bootstrap in "One-time setup".

## How a release flows

1. Every push to `main` makes release-plz open or refresh a PR on a `release-plz-*` branch. It bumps
   each crate changed since its last crates.io release, propagates the new requirements into the
   dependents' `{ version, path }` entries, updates `Cargo.lock`, and writes the crate's
   `CHANGELOG.md`. In CI it runs `cargo-semver-checks`, so a breaking library change becomes a
   minor bump (0.x).
2. Review the PR. Edit versions or changelog text in it if needed — release-plz keeps your edits.
   Changelog entries land under "Other" unless commit messages use conventional prefixes
   (`feat:`, `fix:`, `feat!:`).
3. Merging it runs the `release` job (after approval on the `release` environment). It publishes
   the changed crates leaves-first and tags them:
   - libraries: `<crate>-v<version>`, no GitHub Release
   - `sologger`: `v<version>` plus a GitHub Release, which then gets
     - Linux binaries from `release-binaries.yml`: `sologger-{logstash,otel}-v<version>-{x86_64,aarch64}-unknown-linux-gnu.tar.gz` and `SHA256SUMS`
     - Docker images from `docker.yml`: `brytelands/sologger-{logstash,otel}:{<version>,latest}` for linux/amd64 + linux/arm64

Ordinary pushes to `main` never publish (`release_always = false`).

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

Members depend on each other as `{ version = "x", path = "../..." }`. The path gives local builds
the sibling source and gives cargo/release-plz the publish order; the version is what the
published package keeps. Never replace these with bare versions plus a `[patch]` — that once made
`cargo publish --workspace` order the crates alphabetically (`sologger` first) and fail.

## One-time setup (repository owner, outside the code)

- **First publish of `sologger`.** crates.io trusted publishing cannot create a crate. For the
  first release, add a `CARGO_REGISTRY_TOKEN` repository secret (crates.io token with
  `publish-new` + `publish-update`, scoped to the six sologger crates). The workflow uses it when
  present and falls back to trusted publishing when absent.
- **Trusted publishing** (after `sologger` exists): on crates.io, for each of the six crates,
  Settings → Trusted Publishing → GitHub: repository `brytelands/sologger`, workflow
  `release-plz.yml`, environment `release`. Then delete the `CARGO_REGISTRY_TOKEN` secret.
- **`release` environment** (Settings → Environments): add yourself as a required reviewer, so
  nothing publishes without an approval click.
- **Actions settings** → Workflow permissions: allow GitHub Actions to create pull requests.
- **Optional `RELEASE_PLZ_TOKEN`** secret (fine-grained PAT: contents + pull requests read/write):
  PRs opened with the default token do not trigger CI, so without it the release PR has no checks.
- `DOCKERHUB_USERNAME` / `DOCKERHUB_TOKEN` secrets (already used by `docker.yml`).

## Checking a release locally

```bash
cargo publish --workspace --dry-run --no-verify --allow-dirty  # prints the publish order
cargo package --workspace --allow-dirty                        # builds each crate as packaged
```

`release-plz update` run in a scratch clone shows the version bumps the PR would make.

Each crate's `exclude` list drops `/tests`, `/benches`, `/docs` from the package; the resulting
"ignoring test/benchmark" warnings are expected.

## Re-running pieces

- Binaries for an existing release: Actions → Release Binaries → Run workflow, tag `v<version>`
  (replaces the assets).
- Docker images for an existing tag: Actions → Docker Publish → Run workflow on that tag (also
  moves `latest`).
- A failed `release` job can be re-run from the Actions UI; release-plz skips crates whose version
  is already on crates.io.
- A release run that is stuck (e.g. stranded `queued` by a GitHub outage, so it can be neither
  re-run nor cancelled): Actions → Release-plz → Run workflow on `main`. The manual run skips the
  merged-release-PR check and goes straight to the `release` job (still behind the environment
  approval); reject the stuck run if it ever starts. Do not open a dummy `release-plz-*` PR to
  re-trigger — that was the workaround before this trigger existed.

## After a release

- If `sologger_log_context` or `sologger_log_transformer_wasm` changed, run the `wasm-sync` skill so
  the committed dev copy under `sologger-ui/public/` matches (the Pages deploy rebuilds from source
  regardless).

## Report

List each crate published with its version, the GitHub Release and its assets, the Docker tags, and
anything that failed or was skipped.
