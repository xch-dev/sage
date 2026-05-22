import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  rmSync,
  statSync,
} from 'node:fs';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');

const runtimeSrc = join(repoRoot, 'builtin-apps/src/runtime');
const sandboxTestSrc = join(repoRoot, 'builtin-apps/src/sandbox-test');

const outRoot = join(repoRoot, 'builtin-apps/build/dist');
const runtimeOut = join(outRoot, 'runtime');
const testOut = join(outRoot, 'sandbox-test');

const userSdkDist = join(repoRoot, 'packages/sage-app-sdk/dist');
const pnpm = process.platform === 'win32' ? 'pnpm.cmd' : 'pnpm';

function copyDirFresh(src, dst) {
  rmSync(dst, { recursive: true, force: true });
  mkdirSync(dst, { recursive: true });
  cpSync(src, dst, { recursive: true });
}

function copyRuntimeBridge(outDir) {
  cpSync(join(userSdkDist, 'runtime-bridge.js'), join(outDir, 'bridge.js'));
  cpSync(join(userSdkDist, 'index.js'), join(outDir, 'sdk.js'));
}

function finalizeManifest(source, dist) {
  execFileSync(
    pnpm,
    [
      'exec',
      'sage-app',
      'finalize-manifest',
      '--source',
      source,
      '--dist',
      dist,
    ],
    { stdio: 'inherit', cwd: repoRoot },
  );
}

function buildRuntimeApp(name) {
  const src = join(runtimeSrc, name);
  const out = join(runtimeOut, name);

  copyDirFresh(src, out);
  copyRuntimeBridge(out);
  finalizeManifest(join(src, 'sage-manifest.json'), out);
}

function buildSandboxTestVariant({
  sourceDirName,
  outDirName,
  manifestFileName,
}) {
  const shared = join(sandboxTestSrc, '_shared');
  const src = join(sandboxTestSrc, sourceDirName);
  const out = join(testOut, outDirName);

  rmSync(out, { recursive: true, force: true });
  mkdirSync(out, { recursive: true });

  cpSync(shared, out, { recursive: true });
  cpSync(src, out, { recursive: true });

  copyRuntimeBridge(out);
  finalizeManifest(join(src, manifestFileName), out);
}

if (!existsSync(userSdkDist)) {
  throw new Error(`missing user SDK dist at ${userSdkDist}`);
}

mkdirSync(runtimeOut, { recursive: true });
mkdirSync(testOut, { recursive: true });

for (const name of readdirSync(runtimeSrc)) {
  const src = join(runtimeSrc, name);

  if (!statSync(src).isDirectory()) {
    continue;
  }

  console.log(`\n==> Building builtin runtime app: ${name}`);
  buildRuntimeApp(name);
}

const sandboxTests = [
  {
    sourceDirName: 'sage-storage-isolation',
    outDirName: 'sage-storage-isolation-persistent',
    manifestFileName: 'sage-manifest.persistent.json',
  },
  {
    sourceDirName: 'sage-storage-isolation',
    outDirName: 'sage-storage-isolation-incognito',
    manifestFileName: 'sage-manifest.incognito.json',
  },
  {
    sourceDirName: 'storage-persistence',
    outDirName: 'storage-persistence-persistent',
    manifestFileName: 'sage-manifest.persistent.json',
  },
  {
    sourceDirName: 'storage-persistence',
    outDirName: 'storage-persistence-incognito',
    manifestFileName: 'sage-manifest.incognito.json',
  },
  {
    sourceDirName: 'storage-persistence',
    outDirName: 'storage-clear-persistent',
    manifestFileName: 'sage-manifest.persistent.json',
  },
  {
    sourceDirName: 'network-allow-a',
    outDirName: 'network-allow-a',
    manifestFileName: 'sage-manifest.json',
  },
  {
    sourceDirName: 'network-allow-b',
    outDirName: 'network-allow-b',
    manifestFileName: 'sage-manifest.json',
  },
];

for (const test of sandboxTests) {
  console.log(`\n==> Building builtin sandbox test: ${test.outDirName}`);
  buildSandboxTestVariant(test);
}
