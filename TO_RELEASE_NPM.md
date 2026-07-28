# Grok OSS npm release — remaining human step

## Done in-repo (no token required)

- Product binary **`grok-oss`** (`cargo build -p xai-grok-pager-bin --bin grok-oss`)
- Default home **`~/.grok-oss`** (`GROK_OSS_HOME` / legacy `GROK_HOME`)
- Install script **`./scripts/install-grok-oss.sh`**
- npm packages under **`npm/`**:
  - `@brasalabs/grok-oss` (meta, bin `grok-oss`, postinstall)
  - platform optionalDeps (5): `darwin-arm64`, `darwin-x64`, `linux-arm64`,
    `linux-x64`, `win32-x64` (`win32-arm64` deferred until CI builds it)
- Local smoke: `./npm/scripts/pack-local.sh` (no publish, no token)
- CI: `.github/workflows/ci-grok-oss.yml` (build + pack, no publish)
- Release: `.github/workflows/release-grok-oss.yml` (matrix build, artifacts, pack)
  - Job **`publish-npm`** runs on tags / dispatch; **skips cleanly** without
    `NPM_TOKEN`, publishes when the secret is present

## Remaining (you)

1. Ensure npm org **`brasalabs`** exists and your account can publish
   `@brasalabs/*`.
2. Create an automation token with publish access.
3. Add GitHub repository secret **`NPM_TOKEN`** on `nonexphere/grok-build`
   (value = token).
4. Tag and push to the fork, e.g.
   `git tag grok-oss-v0.2.102 && git push fork grok-oss-v0.2.102`
   **or** Actions → `release-grok-oss` → workflow_dispatch with
   `publish_npm: true` (on a branch that has the workflows).

Until step 3–4, **no code work is blocked** — only registry auth.
