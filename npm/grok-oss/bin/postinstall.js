#!/usr/bin/env node
// Runs once after npm install/update. Reads the grok-oss binary from the
// matching per-platform optional dependency (@brasalabs/grok-oss-<platform>)
// and installs it to ~/.grok-oss/bin/ using versioned filenames:
//
//   Unix:    grok-oss-<version>  +  grok-oss  (symlink)
//   Windows: grok-oss-<version>.exe  +  grok-oss.exe  (copy)
//
// Override home with GROK_OSS_HOME (preferred) or GROK_HOME.
const path = require('path');
const fs = require('fs');
const os = require('os');
const zlib = require('zlib');

const SCOPE = '@brasalabs/grok-oss';
const BIN_NAME = 'grok-oss';

const productHome =
    process.env.GROK_OSS_HOME ||
    process.env.GROK_HOME ||
    path.join(os.homedir(), '.grok-oss');
const CANONICAL_DIR = path.join(productHome, 'bin');

const key = `${process.platform}-${process.arch}`;
const SUPPORTED = new Set([
    'darwin-arm64',
    'darwin-x64',
    'linux-x64',
    'linux-arm64',
    'win32-x64',
    'win32-arm64',
]);
if (!SUPPORTED.has(key)) {
    console.error(`${SCOPE}: unsupported platform ${key}`);
    process.exit(0);
}

function resolvePlatformPackageDir() {
    const platformPkg = `${SCOPE}-${key}`;
    try {
        return path.dirname(require.resolve(`${platformPkg}/package.json`));
    } catch {
        return null;
    }
}

let version;
try {
    version = require('../package.json').version;
} catch {}
if (!version) {
    console.error(`${SCOPE}: unable to determine version`);
    process.exit(0);
}

const IS_WINDOWS = process.platform === 'win32';
const EXE = IS_WINDOWS ? '.exe' : '';

fs.mkdirSync(CANONICAL_DIR, { recursive: true });

function installBinary(binName, sourceDir, vendorSubpath) {
    const brPath = path.join(sourceDir, 'bin', vendorSubpath + '.br');
    const rawPath = path.join(sourceDir, 'bin', vendorSubpath);
    let vendoredBinPath;
    if (fs.existsSync(brPath)) {
        const compressed = fs.readFileSync(brPath);
        const decompressed = zlib.brotliDecompressSync(compressed);
        vendoredBinPath = rawPath;
        fs.writeFileSync(vendoredBinPath, decompressed);
        if (!IS_WINDOWS) fs.chmodSync(vendoredBinPath, 0o755);
        try {
            fs.unlinkSync(brPath);
        } catch {}
    } else if (fs.existsSync(rawPath)) {
        vendoredBinPath = rawPath;
    } else {
        console.error(`${SCOPE}: missing binary at ${brPath}`);
        return false;
    }

    const versionedName = `${binName}-${version}${EXE}`;
    const versionedPath = path.join(CANONICAL_DIR, versionedName);
    const canonicalName = `${binName}${EXE}`;
    const canonicalPath = path.join(CANONICAL_DIR, canonicalName);

    if (!fs.existsSync(versionedPath)) {
        const tmpPath = versionedPath + `.tmp.${process.pid}`;
        try {
            fs.copyFileSync(vendoredBinPath, tmpPath);
            if (!IS_WINDOWS) fs.chmodSync(tmpPath, 0o755);
            fs.renameSync(tmpPath, versionedPath);
        } finally {
            try {
                fs.unlinkSync(tmpPath);
            } catch {}
        }
    }

    if (IS_WINDOWS) {
        try {
            fs.copyFileSync(versionedPath, canonicalPath);
        } catch (e) {
            console.error(`${SCOPE}: failed to install ${canonicalName}: ${e.message}`);
            return false;
        }
    } else {
        try {
            try {
                fs.unlinkSync(canonicalPath);
            } catch {}
            fs.symlinkSync(versionedName, canonicalPath);
        } catch (e) {
            console.error(`${SCOPE}: failed to symlink ${canonicalName}: ${e.message}`);
            return false;
        }
    }
    console.log(`${SCOPE}: installed ${canonicalPath}`);
    return true;
}

const platformDir = resolvePlatformPackageDir();
if (!platformDir) {
    // Local pack without optional deps: leave a placeholder note; CI attaches binaries.
    console.warn(
        `${SCOPE}: platform package @brasalabs/grok-oss-${key} not installed ` +
            '(npm --no-optional or local pack without assemble). Skipping binary install.'
    );
    process.exit(0);
}

const ok = installBinary(BIN_NAME, platformDir, BIN_NAME + EXE);
process.exit(ok ? 0 : 0);
