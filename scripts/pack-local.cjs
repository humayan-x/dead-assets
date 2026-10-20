'use strict';
const { spawnSync } = require('node:child_process');
const { mkdtempSync, mkdirSync, cpSync, writeFileSync, rmSync, existsSync } = require('node:fs');
const { tmpdir } = require('node:os');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const platform = `${process.platform}-${process.arch}`;
if (!existsSync(path.join(root, 'native', platform))) throw new Error('Run npm run build:native first');
const staging = mkdtempSync(path.join(tmpdir(), 'dead-assets-npm-'));
try {
  const pkg = { ...require('../package.json'), os: [process.platform], cpu: [process.arch] };
  if (process.platform === 'linux') pkg.libc = ['glibc'];
  delete pkg.scripts;
  writeFileSync(path.join(staging, 'package.json'), JSON.stringify(pkg, null, 2) + '\n');
  for (const file of ['bin', 'README.md', 'LICENSE-MIT', 'LICENSE-APACHE', 'THIRD-PARTY-LICENSES.txt', 'CONTRIBUTING.md', 'SECURITY.md', 'CODE_OF_CONDUCT.md', 'CHANGELOG.md', 'docs/RELEASING.md', 'examples/README.md']) {
    mkdirSync(path.dirname(path.join(staging, file)), { recursive: true });
    cpSync(path.join(root, file), path.join(staging, file), { recursive: true });
  }
  cpSync(path.join(root, 'native', platform), path.join(staging, 'native', platform), { recursive: true });
  const out = path.join(root, 'dist', platform);
  mkdirSync(out, { recursive: true });
  // npm_execpath is provided by npm run on all supported platforms.
  const result = spawnSync(process.execPath, [process.env.npm_execpath, 'pack', '--pack-destination', out], { cwd: staging, stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exitCode = result.status ?? 1;
} finally {
  rmSync(staging, { recursive: true, force: true });
}
