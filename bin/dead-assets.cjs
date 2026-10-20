#!/usr/bin/env node
'use strict';

const { spawn } = require('node:child_process');
const { existsSync, writeSync } = require('node:fs');
function fail(message) {
  writeSync(2, message + '\n');
  process.exit(1);
}
const path = require('node:path');

const supported = new Set(['linux-x64', 'linux-arm64', 'darwin-x64', 'darwin-arm64', 'win32-x64']);
const platform = `${process.platform}-${process.arch}`;
if (!supported.has(platform)) {
  fail(`dead-assets: unsupported platform ${platform}. Install from source with Cargo instead.`);
}
if (process.platform === 'linux' && !process.report.getReport().header.glibcVersionRuntime) {
  fail('dead-assets: this npm build requires glibc Linux; for Alpine/musl, install from source with Cargo.');
}
const executable = path.join(__dirname, '..', 'native', platform, process.platform === 'win32' ? 'dead-assets.exe' : 'dead-assets');
if (!existsSync(executable)) {
  fail(`dead-assets: binary missing for ${platform}. Reinstall the package or install from source with Cargo.`);
}

// No shell: preserve literal arguments, working directory, and interactive stdio.
const child = spawn(executable, process.argv.slice(2), { stdio: 'inherit' });
const handlers = new Map();
for (const signal of ['SIGINT', 'SIGTERM']) {
  const handler = () => child.kill(signal);
  handlers.set(signal, handler);
  process.on(signal, handler);
}
function detach() {
  for (const [signal, handler] of handlers) process.removeListener(signal, handler);
}
child.on('error', (error) => {
  detach();
  console.error(`dead-assets: could not start ${platform} binary: ${error.message}`);
  process.exitCode = 1;
});
child.on('exit', (code, signal) => {
  detach();
  if (signal && process.platform !== 'win32') {
    process.kill(process.pid, signal);
  } else {
    process.exitCode = code ?? 1;
  }
});
