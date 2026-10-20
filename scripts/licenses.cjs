'use strict';
const { readdirSync, readFileSync, writeFileSync } = require('node:fs');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
// Consume Cargo JSON from stdin; a failed metadata command produces invalid/empty JSON.
const metadata = JSON.parse(readFileSync(0, 'utf8'));
const sections = ['Third-party dependency licenses\nGenerated from the dependencies locked in Cargo.lock.\n'];
for (const pkg of metadata.packages.sort((a, b) => a.name < b.name ? -1 : a.name > b.name ? 1 : a.version.localeCompare(b.version))) {
  if (metadata.workspace_members.includes(pkg.id)) continue;
  const directory = path.dirname(pkg.manifest_path);
  const files = readdirSync(directory, { withFileTypes: true })
    .filter((entry) => entry.isFile() && /^(LICENSE|COPYING|NOTICE|UNLICENSE)/i.test(entry.name))
    .map((entry) => entry.name).sort();
  if (pkg.license_file && !files.includes(pkg.license_file)) files.push(pkg.license_file);
  if (files.length === 0) throw new Error(`No license text found for ${pkg.name}; review manually before release`);
  sections.push(`\n${'='.repeat(72)}\n${pkg.name} ${pkg.version} (${pkg.license ?? 'see license file'})\n`);
  for (const file of files) sections.push(`\n--- ${file} ---\n${readFileSync(path.join(directory, file), 'utf8')}`);
}
writeFileSync(path.join(root, 'THIRD-PARTY-LICENSES.txt'), sections.join('\n'));
console.log('Updated THIRD-PARTY-LICENSES.txt; review dependency license changes before release.');
