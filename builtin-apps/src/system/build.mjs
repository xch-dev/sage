import { execFileSync } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';

const packageRoot = resolve(import.meta.dirname);
const appsRoot = join(packageRoot, 'apps');
const outRoot = resolve(packageRoot, '../../build/dist/system');

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';

// 👇 NEW: read CLI args
const onlyApps = process.argv.slice(2); // e.g. ["task-manager"]

function listAllApps() {
  return readdirSync(appsRoot)
    .map((name) => ({ name, dir: join(appsRoot, name) }))
    .filter((entry) => statSync(entry.dir).isDirectory());
}

const apps =
  onlyApps.length > 0
    ? listAllApps().filter((app) => onlyApps.includes(app.name))
    : listAllApps();

if (apps.length === 0) {
  console.log('[system-apps] no apps to build');
  process.exit(0);
}

for (const app of apps) {
  console.log(`\n==> Building system app: ${app.name}`);

  execFileSync(
    pnpm,
    ['exec', 'tsc', '--noEmit', '--project', join(app.dir, 'tsconfig.json')],
    {
      stdio: 'inherit',
      cwd: packageRoot,
    },
  );

  execFileSync(
    pnpm,
    ['exec', 'vite', 'build', '--config', join(app.dir, 'vite.config.ts')],
    {
      stdio: 'inherit',
      cwd: packageRoot,
    },
  );

  execFileSync(
    pnpm,
    [
      'exec',
      'sage-app',
      'finalize-manifest',
      '--source',
      join(app.dir, 'sage-manifest.json'),
      '--dist',
      join(outRoot, app.name),
    ],
    {
      stdio: 'inherit',
      cwd: packageRoot,
    },
  );
}
