# Grok OSS npm release — remaining human step

## Done in-repo (no token required)

- Product binary **`grok-oss`** (`cargo build -p xai-grok-pager-bin --bin grok-oss`)
- Default home **`~/.grok-oss`** (`GROK_OSS_HOME` / legacy `GROK_HOME`)
- Install script **`./scripts/install-grok-oss.sh`**
- npm packages under **`npm/`**:
  - `@brasalabs/grok-oss` (meta, bin `grok-oss`, postinstall)
  - `@brasalabs/grok-oss-<platform>-<arch>` optional deps
- CI: `.github/workflows/ci-grok-oss.yml` (build + pack, no publish)
- Release: `.github/workflows/release-grok-oss.yml` (matrix build, artifacts, pack)
  - Job **`publish-npm`** runs **only** when secret `NPM_TOKEN` is set

## Remaining (you)

1. Ensure npm org **`brasalabs`** exists and your account can publish
   `@brasalabs/*`.
2. Create an automation token with publish access.
3. Add GitHub repository secret **`NPM_TOKEN`** (value = token).
4. Tag and push, e.g. `git tag grok-oss-v0.2.102 && git push origin grok-oss-v0.2.102`
   **or** run workflow_dispatch with `publish_npm: true`.

Until step 3–4, **no code work is blocked** — only registry auth.
