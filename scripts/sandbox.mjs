// Starts a throwaway Sage sandbox for this worktree: two wallets on a simulated chain, served to the
// UI running in a normal browser. See docs/sandbox.md.

import { execFileSync } from 'node:child_process';
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
import crossSpawn from 'cross-spawn';
import { runCommand, signalProcessTree } from './run-command.mjs';

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

/** The command line of a running process, or `null` if there isn't one with this ID. */
function commandLine(pid) {
  try {
    return execFileSync('ps', ['-o', 'command=', '-p', String(pid)], {
      encoding: 'utf8',
    });
  } catch {
    return null;
  }
}

/** The launcher's own entry in `run.json`, so another run can tell that this one is still going. */
const launcherMatch = 'scripts/sandbox.mjs';

function isRunning({ pid, match }) {
  return commandLine(pid)?.includes(match) ?? false;
}

/**
 * Stops whatever a previous run left behind if it was killed without cleaning up, and refuses to
 * start while another run in this worktree is still going. Process IDs are reused, so a process is
 * only treated as the sandbox's if it's still running the command the sandbox started. There's no
 * `ps` to check with on Windows, so leftovers are left alone there.
 */
function stopLeftovers() {
  const run = readJson(paths.run);

  if (isWindows || !run) return;

  if (run.launcher && isRunning(run.launcher)) {
    console.error(
      `[sandbox] A sandbox is already running in this worktree. See ${paths.instance}.`,
    );
    // Exit without the usual cleanup, which would remove the running sandbox's files.
    process.exit(1);
  }

  for (const leftover of run.processes ?? []) {
    if (!isRunning(leftover)) continue;

    try {
      process.kill(-leftover.pid, 'SIGKILL');
    } catch (error) {
      if (error?.code !== 'ESRCH') throw error;
    }
  }

  rmSync(paths.run, { force: true });
}

function recordProcesses() {
  writeJson(paths.run, {
    launcher: { pid: process.pid, match: launcherMatch },
    processes: children.map((child) => ({
      pid: child.pid,
      match: child.match,
    })),
  });
}

/** Resolves to the port that was free to listen on, or `null` if `port` is taken. */
function listenOn(port) {
  return new Promise((resolve) => {
    const server = createServer();
    server.once('error', () => resolve(null));
    server.listen(port, '127.0.0.1', () => {
      const { port: bound } = server.address();
      server.close(() => resolve(bound));
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
      (Number.isInteger(port) && (await listenOn(port))) || (await listenOn(0));
  }

  writeJson(paths.ports, ports);
  return ports;
}

/**
 * Starts a long-running child with its output going to `logPath`. `match` is a part of its command
 * line that identifies it, so a later run can tell whether a stale process ID is still this child.
 */
function spawnLogged(label, command, args, logPath, env = {}) {
  const output = createWriteStream(logPath);
  const child = crossSpawn(command, args, {
    cwd: repoRoot,
    detached: !isWindows,
    env: { ...process.env, ...env },
    stdio: ['ignore', 'pipe', 'pipe'],
  });

  child.stdout.pipe(output);
  child.stderr.pipe(output);

  Object.assign(child, {
    label,
    logPath,
    match: args.join(' '),
    done: false,
    exited: new Promise((resolve) => {
      child.once('close', () => {
        child.done = true;
        resolve(child);
      });
    }),
  });

  children.push(child);
  recordProcesses();
  return child;
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

function exitedMessage(child) {
  return `The ${child.label} stopped unexpectedly:\n${tail(child.logPath)}`;
}

/** Polls `check` until it returns a value, failing if any child exits or time runs out. */
async function waitFor(child, timeoutMs, check) {
  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    const exited = children.find((other) => other.done);
    if (exited) throw new Error(exitedMessage(exited));

    const result = await check();
    if (result) return result;
    await wait(250);
  }

  throw new Error(
    `The ${child.label} didn't start in time:\n${tail(child.logPath)}`,
  );
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

  const allExited = Promise.all(children.map((child) => child.exited));

  for (const child of children) {
    if (!child.done) signalProcessTree(child, 'SIGTERM');
  }

  await Promise.race([allExited, wait(2000)]);

  for (const child of children) {
    if (!child.done) signalProcessTree(child, 'SIGKILL');
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
  // Claim the worktree before the build, so a second run started meanwhile sees this one.
  recordProcesses();
  rmSync(paths.instance, { force: true });
  rmSync(paths.backendReady, { force: true });

  log('Building the sandbox backend...');
  await runCommand('cargo', ['build', '--package', 'sage-sandbox'], {
    cwd: repoRoot,
    stdio: 'inherit',
  });

  const ports = await pickPorts();
  const binary = join(
    repoRoot,
    'target',
    'debug',
    isWindows ? 'sage-sandbox.exe' : 'sage-sandbox',
  );

  log('Starting the simulator and wallets...');
  const backend = spawnLogged(
    'backend',
    binary,
    ['--dir', sandboxDir, '--port', String(ports.bridge)],
    paths.backendLog,
  );

  const ready = await waitFor(backend, 120_000, () =>
    readJson(paths.backendReady),
  );

  const bridgeUrl = `http://127.0.0.1:${ready.port}`;

  log('Starting the UI...');
  const vite = spawnLogged(
    'Vite dev server',
    'pnpm',
    ['exec', 'vite', '--port', String(ports.app)],
    paths.viteLog,
    { SAGE_SANDBOX_BRIDGE: bridgeUrl },
  );

  const appOrigin = `http://127.0.0.1:${ports.app}`;
  await waitFor(vite, 60_000, () => isServing(appOrigin));

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

  const exited = await Promise.race(children.map((child) => child.exited));

  if (!stopping) {
    console.error(`[sandbox] ${exitedMessage(exited)}`);
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
