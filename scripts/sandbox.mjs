// Starts a throwaway Sage sandbox for this worktree: two wallets on a simulated chain, served to the
// UI running in a normal browser. See docs/sandbox.md.

import { spawn } from 'node:child_process';
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { createServer } from 'node:net';
import { dirname, join, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const sandboxDir = join(repoRoot, '.sandbox');
const logsDir = join(sandboxDir, 'logs');
const paths = {
  instance: join(sandboxDir, 'instance.json'),
  ports: join(sandboxDir, 'ports.json'),
  run: join(sandboxDir, 'run.json'),
  backendReady: join(sandboxDir, 'backend.json'),
  backendLog: join(logsDir, 'backend.log'),
  viteLog: join(logsDir, 'vite.log'),
};
const isWindows = process.platform === 'win32';
const children = [];
let stopping = false;

function log(message) {
  console.log(`[sandbox] ${message}`);
}

function readJson(path) {
  try {
    return JSON.parse(readFileSync(path, 'utf8'));
  } catch {
    return null;
  }
}

function writeJson(path, value) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`);
}

function wait(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

function killTree(pid, signal) {
  try {
    if (isWindows) {
      spawn('taskkill', ['/pid', String(pid), '/t', '/f'], { stdio: 'ignore' });
    } else {
      process.kill(-pid, signal);
    }
  } catch (error) {
    if (error?.code !== 'ESRCH') throw error;
  }
}

/** Stops whatever a previous run left behind if it was killed without cleaning up. */
function stopLeftovers() {
  const run = readJson(paths.run);
  for (const pid of run?.pids ?? []) killTree(pid, 'SIGKILL');
  rmSync(paths.run, { force: true });
}

function isPortFree(port) {
  return new Promise((resolve) => {
    const server = createServer();
    server.once('error', () => resolve(false));
    server.listen(port, '127.0.0.1', () => server.close(() => resolve(true)));
  });
}

function freePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const { port } = server.address();
      server.close(() => resolve(port));
    });
  });
}

/** Reuses the ports from the last run when they're still free, so URLs stay the same. */
async function pickPorts() {
  const previous = readJson(paths.ports) ?? {};
  const ports = {};

  for (const name of ['app', 'bridge']) {
    const port = previous[name];
    ports[name] =
      Number.isInteger(port) && (await isPortFree(port))
        ? port
        : await freePort();
  }

  writeJson(paths.ports, ports);
  return ports;
}

function spawnLogged(command, args, logPath, env = {}) {
  const output = createWriteStream(logPath);
  const child = spawn(command, args, {
    cwd: repoRoot,
    detached: !isWindows,
    env: { ...process.env, ...env },
    shell: isWindows,
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  child.stdout.pipe(output);
  child.stderr.pipe(output);
  children.push(child);
  return child;
}

function runInherited(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd: repoRoot,
      shell: isWindows,
      stdio: 'inherit',
    });
    child.once('error', reject);
    child.once('close', (code) =>
      code === 0
        ? resolve()
        : reject(new Error(`${command} ${args.join(' ')} exited with ${code}`)),
    );
  });
}

function tail(path, lines = 30) {
  try {
    return readFileSync(path, 'utf8')
      .trimEnd()
      .split('\n')
      .slice(-lines)
      .join('\n');
  } catch {
    return '';
  }
}

/** Polls `check` until it returns a value, failing if `child` exits or time runs out. */
async function waitFor(name, child, logPath, timeoutMs, check) {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    if (child.exitCode !== null || child.signalCode !== null) {
      throw new Error(`${name} exited during startup:\n${tail(logPath)}`);
    }

    const result = await check();
    if (result) return result;
    await wait(250);
  }

  throw new Error(`${name} didn't start in time:\n${tail(logPath)}`);
}

async function isServing(url) {
  try {
    return (await fetch(url)).ok;
  } catch {
    return false;
  }
}

async function stop(exitCode) {
  if (stopping) return;
  stopping = true;

  for (const child of children) {
    if (child.exitCode === null) killTree(child.pid, 'SIGTERM');
  }

  await wait(1000);

  for (const child of children) {
    if (child.exitCode === null) killTree(child.pid, 'SIGKILL');
  }

  rmSync(paths.run, { force: true });
  rmSync(paths.instance, { force: true });
  process.exit(exitCode);
}

async function main() {
  if (!existsSync(join(repoRoot, 'node_modules'))) {
    throw new Error('Run `pnpm install` in this worktree first.');
  }

  mkdirSync(logsDir, { recursive: true });
  stopLeftovers();
  rmSync(paths.instance, { force: true });
  rmSync(paths.backendReady, { force: true });

  log('Building the sandbox backend...');
  await runInherited('cargo', ['build', '--package', 'sage-sandbox']);

  const ports = await pickPorts();
  const binary = join(
    repoRoot,
    'target',
    'debug',
    isWindows ? 'sage-sandbox.exe' : 'sage-sandbox',
  );

  log('Starting the simulator and wallets...');
  const backend = spawnLogged(
    binary,
    ['--dir', sandboxDir, '--port', String(ports.bridge)],
    paths.backendLog,
  );
  writeJson(paths.run, { pids: [backend.pid] });

  const ready = await waitFor(
    'The sandbox backend',
    backend,
    paths.backendLog,
    120_000,
    () => readJson(paths.backendReady),
  );

  const bridgeUrl = `http://127.0.0.1:${ready.port}`;

  log('Starting the UI...');
  const vite = spawnLogged(
    'pnpm',
    ['exec', 'vite', '--port', String(ports.app)],
    paths.viteLog,
    {
      SAGE_SANDBOX_BRIDGE: bridgeUrl,
      SAGE_SANDBOX_VITE_PORT: String(ports.app),
    },
  );
  writeJson(paths.run, { pids: [backend.pid, vite.pid] });

  const appOrigin = `http://127.0.0.1:${ports.app}`;
  await waitFor('Vite', vite, paths.viteLog, 60_000, () =>
    isServing(appOrigin),
  );

  const wallets = Object.fromEntries(
    ready.wallets.map((wallet) => [
      wallet.name,
      {
        url: `http://${wallet.name}.localhost:${ports.app}`,
        bridgeUrl: `${bridgeUrl}/${wallet.name}`,
        fingerprint: wallet.fingerprint,
        mnemonic: wallet.mnemonic,
        address: wallet.address,
      },
    ]),
  );

  writeJson(paths.instance, {
    app: wallets.alice.url,
    bridge: bridgeUrl,
    wallets,
    logs: { backend: paths.backendLog, vite: paths.viteLog },
  });

  const width = Math.max(...Object.keys(wallets).map((name) => name.length));
  console.log('\nSage sandbox is ready.\n');
  for (const [name, wallet] of Object.entries(wallets)) {
    console.log(
      `  ${name.padEnd(width)}  ${wallet.url}  (bridge ${wallet.bridgeUrl})`,
    );
  }
  console.log(`\n  Details: ${paths.instance}`);
  console.log('  Ctrl+C stops the sandbox and discards its data.\n');

  const exited = await Promise.race(
    [backend, vite].map(
      (child) =>
        new Promise((resolve) =>
          child.once('close', (code) => resolve({ child, code })),
        ),
    ),
  );

  if (!stopping) {
    const name = exited.child === backend ? 'backend' : 'Vite';
    const logPath = exited.child === backend ? paths.backendLog : paths.viteLog;
    console.error(
      `[sandbox] The ${name} stopped unexpectedly:\n${tail(logPath)}`,
    );
    await stop(1);
  }
}

process.on('SIGINT', () => void stop(0));
process.on('SIGTERM', () => void stop(143));
process.on('SIGHUP', () => void stop(129));

try {
  await main();
} catch (error) {
  console.error(`[sandbox] ${error.message}`);
  await stop(1);
}
