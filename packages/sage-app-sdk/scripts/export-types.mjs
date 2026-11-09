import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';

const packageDir = process.cwd();
const repoRoot = path.resolve(packageDir, '../..');
const outPath = path.join(packageDir, 'src', 'generated-types.ts');

const stdout = execFileSync(
  'cargo',
  ['run', '-p', 'sage-apps', '--bin', 'export_bridge_types', '--', 'user'],
  {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
  },
);

fs.writeFileSync(outPath, stdout);
console.log(`Wrote ${path.relative(packageDir, outPath)}`);
