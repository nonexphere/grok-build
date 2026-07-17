#!/usr/bin/env node
// Structural verification for @brasalabs/grok-oss layout (no network).
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const metaPath = path.join(root, 'grok-oss', 'package.json');
const meta = JSON.parse(fs.readFileSync(metaPath, 'utf8'));

const errors = [];
if (meta.name !== '@brasalabs/grok-oss') {
    errors.push(`meta.name expected @brasalabs/grok-oss got ${meta.name}`);
}
if (!meta.bin || meta.bin['grok-oss'] !== 'bin/grok-oss') {
    errors.push(`meta.bin.grok-oss expected bin/grok-oss got ${JSON.stringify(meta.bin)}`);
}
const opts = Object.keys(meta.optionalDependencies || {});
if (opts.length < 6) {
    errors.push(`expected ≥6 optionalDependencies, got ${opts.length}`);
}
for (const n of opts) {
    if (!n.startsWith('@brasalabs/grok-oss-')) {
        errors.push(`optionalDependency not platform-scoped: ${n}`);
    }
}
const platforms = [
    'darwin-arm64',
    'darwin-x64',
    'linux-arm64',
    'linux-x64',
    'win32-arm64',
    'win32-x64',
];
for (const p of platforms) {
    const pj = path.join(root, `grok-oss-${p}`, 'package.json');
    if (!fs.existsSync(pj)) {
        errors.push(`missing platform package ${p}`);
        continue;
    }
    const pkg = JSON.parse(fs.readFileSync(pj, 'utf8'));
    if (pkg.name !== `@brasalabs/grok-oss-${p}`) {
        errors.push(`${p}: name ${pkg.name}`);
    }
}
if (!fs.existsSync(path.join(root, 'grok-oss', 'bin', 'postinstall.js'))) {
    errors.push('missing postinstall.js');
}
if (!fs.existsSync(path.join(root, 'grok-oss', 'bin', 'grok-oss'))) {
    errors.push('missing bin/grok-oss shim');
}

if (errors.length) {
    console.error('assert-package-identity FAILED:');
    for (const e of errors) console.error(' -', e);
    process.exit(1);
}
console.log('assert-package-identity OK', meta.name, meta.version, opts.length, 'platforms');
