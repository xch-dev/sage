import { mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { runCommandSync } from './run-command.mjs';

const repoRoot = resolve(import.meta.dirname, '..');
const packageDir = resolve(repoRoot, 'target/release/bundle/arch');
mkdirSync(packageDir, { recursive: true });

runCommandSync('makepkg', ['--syncdeps', '--force', ...process.argv.slice(2)], {
  cwd: resolve(repoRoot, 'src-tauri/arch'),
  env: { ...process.env, PKGDEST: packageDir },
  stdio: 'inherit',
});
