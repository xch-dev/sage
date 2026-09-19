import { spawn, spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const forwardedArgs = process.argv.slice(2);
const interruptGraceMs = 100;
const shutdownGraceMs = 2_000;

if (forwardedArgs[0] === '--') {
  forwardedArgs.shift();
}

const children = new Set();
let shuttingDown = false;
let shutdownPromise = null;

function spawnManaged(command, args) {
  const child = spawn(command, args, {
    cwd: repoRoot,
    detached: process.platform !== 'win32',
    stdio: 'inherit',
  });

  children.add(child);
  child.once('close', () => children.delete(child));
  return child;
}

function waitForExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return Promise.resolve({
      code: child.exitCode,
      signal: child.signalCode,
    });
  }

  return new Promise((resolve, reject) => {
    child.once('error', reject);
    child.once('close', (code, signal) => resolve({ code, signal }));
  });
}

function signalProcessTree(child, signal) {
  try {
    if (process.platform === 'win32') {
      if (child.exitCode !== null || child.signalCode !== null) {
        return;
      }

      if (signal === 'SIGKILL') {
        spawnSync('taskkill', ['/pid', String(child.pid), '/t', '/f'], {
          stdio: 'ignore',
        });
      } else {
        child.kill(signal);
      }
    } else {
      process.kill(-child.pid, signal);
    }
  } catch (error) {
    if (error?.code !== 'ESRCH') {
      console.error(`[tauri:dev] failed to send ${signal}:`, error);
    }
  }
}

function wait(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}

async function stopChildren(signal, fast) {
  const running = [...children];

  if (running.length === 0) {
    return;
  }

  const allExited = Promise.allSettled(running.map(waitForExit));

  for (const child of running) {
    signalProcessTree(child, signal);
  }

  if (fast) {
    await wait(interruptGraceMs);

    for (const child of running) {
      signalProcessTree(child, 'SIGKILL');
    }

    await allExited;
    return;
  }

  if (
    await Promise.race([
      allExited.then(() => true),
      wait(shutdownGraceMs).then(() => false),
    ])
  ) {
    return;
  }

  for (const child of running) {
    signalProcessTree(child, 'SIGTERM');
  }

  if (
    await Promise.race([
      allExited.then(() => true),
      wait(shutdownGraceMs).then(() => false),
    ])
  ) {
    return;
  }

  for (const child of running) {
    signalProcessTree(child, 'SIGKILL');
  }

  await allExited;
}

function shutdown(signal, exitCode, fast = false) {
  if (shuttingDown) {
    return shutdownPromise;
  }

  shuttingDown = true;
  shutdownPromise = stopChildren(signal, fast).finally(() => {
    process.exitCode = exitCode;
  });
  return shutdownPromise;
}

process.on('SIGINT', () => {
  void shutdown('SIGINT', 130, true);
});
process.on('SIGTERM', () => {
  void shutdown('SIGTERM', 143, true);
});
process.on('SIGHUP', () => {
  void shutdown('SIGHUP', 129, true);
});

async function runPreparation() {
  const child = spawnManaged('pnpm', ['run', 'dev:prepare']);
  const result = await waitForExit(child);

  if (result.code !== 0) {
    throw new Error(
      result.signal
        ? `development preparation stopped with ${result.signal}`
        : `development preparation failed with exit code ${result.code}`,
    );
  }
}

async function main() {
  await runPreparation();

  if (shuttingDown) {
    return;
  }

  const systemApps = spawnManaged('pnpm', ['run', 'dev:system-apps']);
  const tauri = spawnManaged('pnpm', [
    'exec',
    'tauri',
    'dev',
    ...forwardedArgs,
  ]);
  const firstExit = await Promise.race([
    waitForExit(systemApps),
    waitForExit(tauri),
  ]);

  if (shuttingDown) {
    return;
  }

  const exitCode = firstExit.code ?? (firstExit.signal ? 1 : 0);
  await shutdown('SIGTERM', exitCode);
}

try {
  await main();
} catch (error) {
  if (!shuttingDown) {
    console.error('[tauri:dev]', error);
    await shutdown('SIGTERM', 1);
  }
}
