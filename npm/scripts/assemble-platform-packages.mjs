#!/usr/bin/env node
// Assemble @brasalabs/grok-oss-* platform packages for publish / local pack.
//
// Compresses built binaries with brotli into npm/grok-oss-<os>-<arch>/bin/grok-oss.br
// and stamps versions to match the meta package.
//
// Env overrides (CI):
//   GROK_OSS_BIN_LINUX_X64, GROK_OSS_BIN_LINUX_ARM64,
//   GROK_OSS_BIN_DARWIN_X64, GROK_OSS_BIN_DARWIN_ARM64,
//   GROK_OSS_BIN_WIN32_X64, GROK_OSS_BIN_WIN32_ARM64
//   REPO_ROOT (default: repo root from this file)
import fs from 'fs';
import path from 'path';
import { promisify } from 'util';
import zlib from 'zlib';
import { fileURLToPath } from 'url';

const brotliCompress = promisify(zlib.brotliCompress);
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const npmRoot = path.resolve(__dirname, '..');
const repoRoot = process.env.REPO_ROOT || path.resolve(npmRoot, '..');

const META_PKG_JSON = path.join(npmRoot, 'grok-oss', 'package.json');
const meta = JSON.parse(fs.readFileSync(META_PKG_JSON, 'utf8'));
const VERSION = meta.version;
const BIN = 'grok-oss';

const NOTICES_CANDIDATES = [
    path.join(repoRoot, 'crates/codegen/xai-grok-tools/THIRD_PARTY_NOTICES.md'),
    path.join(repoRoot, 'THIRD_PARTY_NOTICES.md'),
];

function ensureDir(p) {
    fs.mkdirSync(path.dirname(p), { recursive: true });
}

async function packPlatform({ platform, arch, envVar, defaultSource }) {
    const pkgDir = path.join(npmRoot, `grok-oss-${platform}-${arch}`);
    const pkgJsonPath = path.join(pkgDir, 'package.json');
    if (!fs.existsSync(pkgJsonPath)) {
        console.error(`[assemble] Missing package at ${pkgDir}`);
        return false;
    }

    const source = process.env[envVar] || defaultSource;
    if (!fs.existsSync(source)) {
        console.error(`[assemble] Missing binary for ${platform}-${arch}: ${source}`);
        console.error(`            Set ${envVar} or build --bin ${BIN}.`);
        return false;
    }

    const subPkg = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
    subPkg.version = VERSION;
    fs.writeFileSync(pkgJsonPath, JSON.stringify(subPkg, null, 4) + '\n');

    const notices = NOTICES_CANDIDATES.find((p) => fs.existsSync(p));
    if (notices) {
        fs.copyFileSync(notices, path.join(pkgDir, 'THIRD_PARTY_NOTICES.md'));
    } else {
        fs.writeFileSync(
            path.join(pkgDir, 'THIRD_PARTY_NOTICES.md'),
            '# Third-party notices\n\nSee the Grok OSS / xAI monorepo source tree.\n'
        );
    }

    const binName = platform.startsWith('win') ? `${BIN}.exe` : BIN;
    ensureDir(path.join(pkgDir, 'bin', binName));
    const raw = fs.readFileSync(source);
    // Always ship raw binary for postinstall fallback / local pack smoke.
    fs.writeFileSync(path.join(pkgDir, 'bin', binName), raw);
    if (!platform.startsWith('win')) fs.chmodSync(path.join(pkgDir, 'bin', binName), 0o755);

    // Brotli is expensive on large debug binaries; default to raw-only for
    // --host-only / GROK_OSS_SKIP_BROTLI=1; full quality in CI release.
    const skipBrotli =
        process.argv.includes('--host-only') || process.env.GROK_OSS_SKIP_BROTLI === '1';
    let brMb = 'skipped';
    if (!skipBrotli) {
        const outBr = path.join(pkgDir, 'bin', `${binName}.br`);
        const quality = Number(process.env.GROK_OSS_BROTLI_QUALITY || '6');
        const compressed = await brotliCompress(raw, {
            params: { [zlib.constants.BROTLI_PARAM_QUALITY]: quality },
        });
        fs.writeFileSync(outBr, compressed);
        brMb = `${(compressed.length / 1048576).toFixed(1)} MB`;
    }

    console.log(
        `[assemble] @brasalabs/grok-oss-${platform}-${arch}@${VERSION}: ` +
            `${(raw.length / 1048576).toFixed(1)} MB raw, brotli=${brMb}`
    );
    return true;
}

const targets = [
    {
        platform: 'linux',
        arch: 'x64',
        envVar: 'GROK_OSS_BIN_LINUX_X64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss'),
    },
    {
        platform: 'linux',
        arch: 'arm64',
        envVar: 'GROK_OSS_BIN_LINUX_ARM64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss'),
    },
    {
        platform: 'darwin',
        arch: 'x64',
        envVar: 'GROK_OSS_BIN_DARWIN_X64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss'),
    },
    {
        platform: 'darwin',
        arch: 'arm64',
        envVar: 'GROK_OSS_BIN_DARWIN_ARM64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss'),
    },
    {
        platform: 'win32',
        arch: 'x64',
        envVar: 'GROK_OSS_BIN_WIN32_X64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss.exe'),
    },
    {
        platform: 'win32',
        arch: 'arm64',
        envVar: 'GROK_OSS_BIN_WIN32_ARM64',
        defaultSource: path.join(repoRoot, 'target/release/grok-oss.exe'),
    },
];

const onlyHost = process.argv.includes('--host-only');
const hostKey = `${process.platform}-${process.arch}`;

let ok = 0;
let fail = 0;
for (const t of targets) {
    if (onlyHost && `${t.platform}-${t.arch}` !== hostKey) {
        console.log(`[assemble] skip ${t.platform}-${t.arch} (--host-only)`);
        continue;
    }
    // Prefer debug bin for local host-only if release missing.
    if (onlyHost && !process.env[t.envVar]) {
        const debug = path.join(
            repoRoot,
            process.platform === 'win32' ? 'target/debug/grok-oss.exe' : 'target/debug/grok-oss'
        );
        if (fs.existsSync(debug)) t.defaultSource = debug;
    }
    const success = await packPlatform(t);
    if (success) ok++;
    else fail++;
}

// Stamp meta package version is already source of truth.
console.log(`[assemble] done: ${ok} ok, ${fail} failed (meta @brasalabs/grok-oss@${VERSION})`);
process.exit(fail > 0 && ok === 0 ? 1 : 0);
