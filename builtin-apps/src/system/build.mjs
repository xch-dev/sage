import { spawn } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import { join, resolve } from 'node:path';

const packageRoot = resolve(import.meta.dirname);
const appsRoot = join(packageRoot, 'apps');
const outRoot = resolve(packageRoot, '../../build/dist/system');

const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';

const onlyApps = process.argv.slice(2);
const concurrency = Number(process.env.SYSTEM_APPS_CONCURRENCY ?? 4);

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

function run(command, args) {
  return new Promise((resolvePromise, rejectPromise) => {
    const child = spawn(command, args, {
      cwd: packageRoot,
      stdio: 'inherit',
      shell: false,
    });

    child.on('error', rejectPromise);

    child.on('exit', (code, signal) => {
      if (code === 0) {
        resolvePromise();
        return;
      }

      rejectPromise(
        new Error(
          `${command} ${args.join(' ')} failed with ${
            signal ? `signal ${signal}` : `exit code ${code}`
          }`,
        ),
      );
    });
  });
}

async function buildApp(app) {
  console.log(`\n==> Building system app: ${app.name}`);

  await run(pnpm, [
    'exec',
    'tsc',
    '--noEmit',
    '--project',
    join(app.dir, 'tsconfig.json'),
  ]);

  await run(pnpm, [
    'exec',
    'vite',
    'build',
    '--config',
    join(app.dir, 'vite.config.ts'),
  ]);

  await run(pnpm, [
    'exec',
    'sage-app',
    'finalize-manifest',
    '--source',
    join(app.dir, 'sage-manifest.json'),
    '--dist',
    join(outRoot, app.name),
  ]);

  console.log(`\n✓ Finished system app: ${app.name}`);
}

async function runWithConcurrency(items, limit, worker) {
  let index = 0;

  const workers = Array.from(
    { length: Math.min(limit, items.length) },
    async () => {
      while (index < items.length) {
        const item = items[index];
        index += 1;
        await worker(item);
      }
    },
  );

  await Promise.all(workers);
}

try {
  await runWithConcurrency(apps, concurrency, buildApp);
  console.log('\n[system-apps] all builds finished');
} catch (error) {
  console.error('\n[system-apps] build failed');
  console.error(error);
  process.exit(1);
}
