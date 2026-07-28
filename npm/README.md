# Grok OSS npm packages (`@brasalabs/grok-oss`)

Public install (after publish + npm token):

```bash
npm install -g @brasalabs/grok-oss
grok-oss --version
```

Installs the native binary under **`~/.grok-oss/bin/grok-oss`** (override with
`GROK_OSS_HOME`).

## Layout

| Package | Role |
|---------|------|
| `@brasalabs/grok-oss` | Meta package: `bin` → `grok-oss`, postinstall |
| `@brasalabs/grok-oss-<os>-<arch>` | Optional platform binary (brotli + raw) |

## Local pack (no token)

```bash
cargo build -p xai-grok-pager-bin --bin grok-oss
./npm/scripts/pack-local.sh
# tarballs under npm/dist/
```

## Release (CI)

See `.github/workflows/release-grok-oss.yml`. Publish jobs require repository
secret **`NPM_TOKEN`** with publish rights on the `brasalabs` npm org.

## Remaining human step

1. Create/claim npm org `brasalabs` (if needed).
2. Add `NPM_TOKEN` secret to the GitHub repo.
3. Tag a release or run the workflow_dispatch release job.
