import { createHash } from 'node:crypto';
import {
  access,
  mkdir,
  readFile,
  readdir,
  rename,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join, relative, resolve } from 'node:path';
import { runCommand } from './run-command.mjs';

const CACHE_VERSION = 1;
const repoRoot = resolve(import.meta.dirname, '..');
const cachePath = join(
  repoRoot,
  'node_modules',
  '.cache',
  'sage',
  'dev-prepare.json',
);
const ignoredInputNames = new Set([
  '.git',
  'dist',
  'generated-types.ts',
  'node_modules',
  'target',
]);

function fromRoot(path) {
  return resolve(repoRoot, path);
}

async function pathExists(path) {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

async function inputFiles(path) {
  if (!(await pathExists(path))) {
    return [];
  }

  if ((await stat(path)).isFile()) {
    return [path];
  }

  const entries = await readdir(path, { withFileTypes: true });
  const files = [];

  for (const entry of entries.sort((a, b) => a.name.localeCompare(b.name))) {
    if (ignoredInputNames.has(entry.name)) {
      continue;
    }

    const entryPath = join(path, entry.name);

    if (entry.isDirectory()) {
      files.push(...(await inputFiles(entryPath)));
    } else if (entry.isFile()) {
      files.push(entryPath);
    }
  }

  return files;
}

async function fingerprint(paths, dependencies = []) {
  const hash = createHash('sha256');

  for (const dependency of dependencies) {
    hash.update(`dependency:${dependency}\0`);
  }

  for (const input of paths.map(fromRoot).sort()) {
    if (!(await pathExists(input))) {
      hash.update(`missing:${relative(repoRoot, input)}\0`);
      continue;
    }

    const files = (await inputFiles(input)).sort();

    if (files.length === 0) {
      files.push(input);
    }

    for (const file of files) {
      hash.update(`${relative(repoRoot, file)}\0`);
      hash.update(await readFile(file));
      hash.update('\0');
    }
  }

  return hash.digest('hex');
}

async function loadCache() {
  try {
    const cache = JSON.parse(await readFile(cachePath, 'utf8'));

    if (cache.version === CACHE_VERSION && typeof cache.tasks === 'object') {
      return cache;
    }
  } catch {
    // A missing or invalid cache is equivalent to a cold start.
  }

  return { version: CACHE_VERSION, tasks: {} };
}

const cache = await loadCache();

async function saveCache() {
  await mkdir(resolve(cachePath, '..'), { recursive: true });

  const temporaryPath = `${cachePath}.${process.pid}.tmp`;
  await writeFile(temporaryPath, `${JSON.stringify(cache, null, 2)}\n`);
  await rename(temporaryPath, cachePath);
}

async function outputsExist(outputs) {
  return (
    outputs.length > 0 &&
    (
      await Promise.all(outputs.map((output) => pathExists(fromRoot(output))))
    ).every(Boolean)
  );
}

async function runPnpm(args) {
  await runCommand('pnpm', args, {
    cwd: repoRoot,
    stdio: 'inherit',
  });
}

async function runCachedTask({
  name,
  label,
  inputs,
  outputs,
  dependencies = [],
  run,
}) {
  const nextFingerprint = await fingerprint(inputs, dependencies);

  if (cache.tasks[name] === nextFingerprint && (await outputsExist(outputs))) {
    console.log(`[dev:prepare] ${label}: cached`);
    return nextFingerprint;
  }

  console.log(`[dev:prepare] ${label}: building`);
  await run();

  if (!(await outputsExist(outputs))) {
    throw new Error(`${label} completed without producing its expected output`);
  }

  cache.tasks[name] = nextFingerprint;
  return nextFingerprint;
}

const appSdkHash = await runCachedTask({
  name: 'app-sdk',
  label: 'app SDK',
  inputs: [
    'Cargo.lock',
    'crates/sage-apps/Cargo.toml',
    'crates/sage-apps/src',
    'packages/sage-app-sdk/cli',
    'packages/sage-app-sdk/package.json',
    'packages/sage-app-sdk/scripts',
    'packages/sage-app-sdk/src',
    'packages/sage-app-sdk/tsconfig.json',
    'packages/sage-app-sdk/tsup.config.ts',
    'pnpm-lock.yaml',
  ],
  outputs: [
    'packages/sage-app-sdk/dist/index.js',
    'packages/sage-app-sdk/src/generated-types.ts',
  ],
  run: () => runPnpm(['--filter', 'sage-app-sdk', 'build']),
});
await saveCache();

const [, systemSdkHash] = await Promise.all([
  runCachedTask({
    name: 'builtin-static',
    label: 'builtin static apps',
    inputs: [
      'builtin-apps/src/runtime',
      'builtin-apps/src/sandbox-test',
      'scripts/build-builtin-static.mjs',
    ],
    outputs: [
      'builtin-apps/build/dist/runtime',
      'builtin-apps/build/dist/sandbox-test',
    ],
    dependencies: [appSdkHash],
    run: () => runPnpm(['run', 'build:builtin-static']),
  }),
  runCachedTask({
    name: 'system-app-sdk',
    label: 'system app SDK',
    inputs: [
      'Cargo.lock',
      'crates/sage-apps/Cargo.toml',
      'crates/sage-apps/src',
      'packages/sage-system-app-sdk/package.json',
      'packages/sage-system-app-sdk/scripts',
      'packages/sage-system-app-sdk/src',
      'packages/sage-system-app-sdk/tsconfig.build.json',
      'packages/sage-system-app-sdk/tsconfig.json',
      'packages/sage-system-app-sdk/tsup.config.ts',
      'pnpm-lock.yaml',
    ],
    outputs: [
      'packages/sage-system-app-sdk/dist/index.js',
      'packages/sage-system-app-sdk/src/generated-types.ts',
    ],
    dependencies: [appSdkHash],
    run: () => runPnpm(['--filter', 'sage-system-app-sdk', 'build']),
  }),
]);
await saveCache();

const appUiHash = await runCachedTask({
  name: 'app-ui',
  label: 'app UI',
  inputs: [
    'packages/sage-app-ui/package.json',
    'packages/sage-app-ui/src',
    'packages/sage-app-ui/tsconfig.json',
    'packages/sage-app-ui/tsup.config.ts',
    'pnpm-lock.yaml',
  ],
  outputs: ['packages/sage-app-ui/dist/index.js'],
  dependencies: [systemSdkHash],
  run: () => runPnpm(['--filter', 'sage-app-ui', 'build']),
});
await saveCache();

const systemAppsRoot = fromRoot('builtin-apps/src/system/apps');
const systemAppNames = (await readdir(systemAppsRoot, { withFileTypes: true }))
  .filter((entry) => entry.isDirectory())
  .map((entry) => entry.name)
  .sort();
const staleSystemApps = [];
const systemAppHashes = new Map();

for (const appName of systemAppNames) {
  const taskName = `system-app:${appName}`;
  const nextFingerprint = await fingerprint(
    [
      `builtin-apps/src/system/apps/${appName}`,
      'builtin-apps/src/system/build.mjs',
      'builtin-apps/src/system/package.json',
      'builtin-apps/src/system/tailwind.shared.js',
      'builtin-apps/src/system/tsconfig.base.json',
      'builtin-apps/src/system/vite.system-app.config.ts',
      'pnpm-lock.yaml',
    ],
    [appUiHash, systemSdkHash],
  );
  const outputs = [
    `builtin-apps/build/dist/system/${appName}/index.html`,
    `builtin-apps/build/dist/system/${appName}/sage-manifest.json`,
  ];

  systemAppHashes.set(taskName, nextFingerprint);

  if (
    cache.tasks[taskName] !== nextFingerprint ||
    !(await outputsExist(outputs))
  ) {
    staleSystemApps.push(appName);
  }
}

if (staleSystemApps.length > 0) {
  console.log(
    `[dev:prepare] system apps: building ${staleSystemApps.join(', ')}`,
  );
  await runPnpm(['run', 'build:system-apps', '--', ...staleSystemApps]);

  for (const appName of staleSystemApps) {
    const taskName = `system-app:${appName}`;
    cache.tasks[taskName] = systemAppHashes.get(taskName);
  }
} else {
  console.log('[dev:prepare] system apps: cached');
}

await saveCache();
console.log('[dev:prepare] ready');
