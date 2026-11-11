import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';

assert.equal(
  process.argv.length,
  3,
  'Usage: pnpm check:flatpak-package <flatpak-build-directory>',
);
const buildDir = resolve(process.argv[2]);

// Check the runtime users install; SDK-only libraries must not hide missing dependencies.
const runtime = ['build', '--runtime', '--readonly', buildDir];
const libraries = execFileSync(
  'flatpak',
  [...runtime, 'ldd', '-r', '/app/bin/sage-tauri'],
  { encoding: 'utf8' },
);
assert.doesNotMatch(libraries, /not found|undefined symbol/, libraries);
console.log(`Flatpak package checks passed: ${buildDir}`);
