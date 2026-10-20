'use strict';
const { readFileSync, statSync, chmodSync } = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const pkg = require('../package.json');
const cargoVersion = readFileSync(path.join(root, 'Cargo.toml'), 'utf8').match(/^version\s*=\s*"([^"]+)"/m)?.[1];
if (pkg.version !== cargoVersion) throw new Error('package.json and Cargo.toml versions must match');
for (const platform of require('./platforms.json')) {
  const name = platform.startsWith('win32-') ? 'dead-assets.exe' : 'dead-assets';
  const binary = path.join(root, 'native', platform, name);
  if (!statSync(binary, { throwIfNoEntry: false })?.isFile() || statSync(binary).size === 0) {
    throw new Error(`Missing native/${platform}/${name}. Run the npm release workflow for a complete package, or npm run pack:local for a host-only package.`);
  }
  // Artifact downloads do not preserve executable permissions.
  chmodSync(binary, 0o755);
}
console.log(`All platform binaries present for dead-assets@${pkg.version}`);
