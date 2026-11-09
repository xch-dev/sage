import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { runCommandSync } from './run-command.mjs';

assert.equal(
  process.argv.length,
  3,
  'Usage: pnpm package:flatpak <Sage_amd64.deb>',
);
const debPath = resolve(process.argv[2]);
const version = execFileSync('dpkg-deb', ['--field', debPath, 'Version'], {
  encoding: 'utf8',
}).trim();
const arch = execFileSync('dpkg-deb', ['--field', debPath, 'Architecture'], {
  encoding: 'utf8',
}).trim();
assert.equal(arch, 'amd64', 'Flatpak packaging currently supports x86_64');
assert.match(version, /^[0-9A-Za-z.+:~_-]+$/, 'Invalid DEB version');

const repoRoot = resolve(import.meta.dirname, '..');
const workDir = join(repoRoot, 'target/flatpak');
const packageDir = join(repoRoot, 'target/release/bundle/flatpak');
const manifest = 'com.rigidnetwork.sage.json';
const bundle = join(packageDir, `Sage_${version}_x64.flatpak`);
mkdirSync(workDir, { recursive: true });
mkdirSync(packageDir, { recursive: true });
copyFileSync(debPath, join(workDir, 'sage.deb'));
copyFileSync(join(repoRoot, 'LICENSE'), join(workDir, 'LICENSE'));
copyFileSync(
  join(repoRoot, 'src-tauri/flatpak', manifest),
  join(workDir, manifest),
);

const options = { cwd: workDir, stdio: 'inherit' };
runCommandSync(
  'flatpak-builder',
  [
    '--user',
    '--force-clean',
    '--disable-cache',
    '--disable-rofiles-fuse',
    '--arch=x86_64',
    '--repo=repo',
    'build',
    manifest,
  ],
  options,
);
runCommandSync(
  process.execPath,
  [join(repoRoot, 'scripts/check-flatpak-package.mjs'), join(workDir, 'build')],
  options,
);
runCommandSync(
  'flatpak',
  [
    'build-bundle',
    '--arch=x86_64',
    '--runtime-repo=https://dl.flathub.org/repo/flathub.flatpakrepo',
    'repo',
    bundle,
    'com.rigidnetwork.sage',
    'stable',
  ],
  options,
);
console.log(`Flatpak bundle created: ${bundle}`);
