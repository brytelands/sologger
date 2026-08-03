---
name: deploy-ui
description: Deploy or troubleshoot the hosted Sologger UI on GitHub Pages. Use when asked to deploy, publish, or update the live site, check why a deploy did not go out, or verify what the published site is serving.
---

# Deploy sologger-ui to GitHub Pages

Deployment is automated by `.github/workflows/pages.yml`. There is no tracked copy of the built
site and nothing to copy by hand.

## How a deploy happens

A push to `main` touching any of these paths triggers it:

- `sologger-ui/**`
- `sologger-log-transformer-wasm/**`
- `sologger-log-context/**`
- the workflow file itself

The workflow then: builds the WASM transformer from current source (so the site never ships a stale
parser), overlays it into `sologger-ui/public/`, runs `npm ci` + `npx vitest run` (a red test blocks
the deploy), builds with Vite, and publishes `sologger-ui/dist/` with `actions/deploy-pages` to
<https://brytelands.github.io/sologger/>.

So the normal "deploy" procedure is: land the change on `main`. That's it.

## Prerequisite (one-time, repo settings)

The repo's Pages source must be **GitHub Actions** (Settings → Pages → Build and deployment →
Source). If it is still "Deploy from a branch", the workflow runs but its deployments are not what
the site serves. This cannot be changed from the CLI here (no `gh` installed) — it is a web-UI step.

## Manual deploy without a code change

The workflow has `workflow_dispatch`: GitHub → Actions → "Deploy UI to GitHub Pages" → Run
workflow. Use this to republish (e.g. after flipping the Pages source, or to pick up a
`sologger-log-transformer` change, which is not in the trigger paths).

## Verifying before landing

```bash
cd sologger-ui
npx vitest run       # the same gate the workflow runs
npm run preview      # builds and serves the production bundle locally
```

`npm run preview` uses the *committed* WASM package under `public/`; the deployed site uses a fresh
build. If you need the preview to match production exactly after Rust parser changes, run the
`wasm-sync` skill first.

## Troubleshooting a missing or stale deploy

1. Check the run: GitHub → Actions → "Deploy UI to GitHub Pages". A red `vitest` step means the
   deploy was correctly blocked by a failing test.
2. No run at all? The push probably didn't touch a trigger path — `sologger-log-transformer/`
   changes, for example, don't trigger. Use `workflow_dispatch`.
3. Run green but site unchanged? Almost certainly the Pages source setting (see prerequisite), or
   CDN caching — deployments can take a minute or two to propagate.
4. Deep links 404 while the home page works: `vite.config.js` `base` and the router's
   `createWebHistory()` argument must both be `'/sologger/'`.

## Invariants

- `sologger-ui/dist/` and the old root `docs/` directory stay untracked/deleted — never commit
  built output as the site again.
- The workflow must keep building WASM before the Vite build; removing that step reintroduces the
  stale-parser drift this pipeline exists to prevent.
