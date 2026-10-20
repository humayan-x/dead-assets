'use strict';
const { test } = require('node:test');
const assert = require('node:assert/strict');
const { spawnSync } = require('node:child_process');
const { mkdtempSync, mkdirSync, writeFileSync, rmSync, existsSync, cpSync } = require('node:fs');
const { tmpdir } = require('node:os');
const path = require('node:path');
const root = path.resolve(__dirname, '..');
const cli = path.join(root, 'bin', 'dead-assets.cjs');
function run(args, options = {}) {
  return spawnSync(process.execPath, [cli, ...args], { encoding: 'utf8', ...options });
}

test('launcher runs native executable with matching package version', () => {
  const result = run(['--version']);
  assert.equal(result.status, 0, result.stderr);
  assert.ok(result.stdout.includes(require('../package.json').version));
});

test('preserves arguments containing spaces, cwd, JSON, and dry-run behavior', () => {
  const workspace = mkdtempSync(path.join(tmpdir(), 'dead assets npm '));
  try {
    mkdirSync(path.join(workspace, 'public'));
    writeFileSync(path.join(workspace, 'public', 'unused.png'), 'image');
    for (const args of [['--json', '--dry-run'], ['--json', '--dry-run', workspace]]) {
      const result = run(args, { cwd: workspace });
      assert.equal(result.status, 0, result.stderr);
      const report = JSON.parse(result.stdout);
      assert.equal(report.audit.dead_assets, 1);
      assert.equal(report.cleanup.is_dry_run, true);
    }
    assert.ok(existsSync(path.join(workspace, 'public', 'unused.png')));
    const error = run(['--not-a-real-option']);
    assert.notEqual(error.status, 0);
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
});

test('missing native executable gives actionable error', () => {
  const workspace = mkdtempSync(path.join(tmpdir(), 'dead-assets-missing-'));
  try {
    mkdirSync(path.join(workspace, 'bin'));
    cpSync(cli, path.join(workspace, 'bin', 'dead-assets.cjs'));
    const result = spawnSync(process.execPath, [path.join(workspace, 'bin', 'dead-assets.cjs')], { encoding: 'utf8' });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /binary missing/);
  } finally {
    rmSync(workspace, { recursive: true, force: true });
  }
});
