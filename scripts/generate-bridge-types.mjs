import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');

execFileSync('cargo', ['run', '-p', 'sage-apps', '--bin', 'export_bridge_types'], {
  cwd: repoRoot,
  stdio: ['ignore', 'ignore', 'inherit'],
});
