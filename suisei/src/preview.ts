#!/usr/bin/env node
import { spawn } from 'node:child_process';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

async function preview() {
  console.log('🚀 Starting fn0 preview server...\n');

  const cwd = process.cwd();
  const staticDir = resolve(cwd, './dist/client');
  const wasmPath = resolve(cwd, './dist/server/component.wasm');

  // fn0-cli path (debug build)
  const cliBin = resolve(
    dirname(fileURLToPath(import.meta.url)),
    '../../cli/target/debug/fn0-cli'
  );

  const port = process.env.PORT || '4321';

  const fn0Process = spawn(
    cliBin,
    ['local', '--port', port, '--static-dir', staticDir],
    {
      stdio: 'inherit',
      cwd,
    }
  );

  fn0Process.on('error', (err) => {
    console.error('Failed to start fn0-cli:', err);
    process.exit(1);
  });

  fn0Process.on('exit', (code) => {
    process.exit(code || 0);
  });

  // Cleanup on SIGINT
  process.on('SIGINT', () => {
    fn0Process.kill('SIGTERM');
  });
}

preview();
