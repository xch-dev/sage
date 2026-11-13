import { spawnSync } from 'node:child_process';
import process from 'node:process';
import crossSpawn from 'cross-spawn';

function commandLabel(command, args) {
  return [command, ...args].join(' ');
}

function commandError(command, args, code, signal) {
  const reason = signal ? `signal ${signal}` : `exit code ${code}`;
  return new Error(`${commandLabel(command, args)} failed with ${reason}`);
}

export function runCommand(command, args, options = {}) {
  return new Promise((resolve, reject) => {
    const child = crossSpawn(command, args, options);

    child.once('error', reject);
    child.once('close', (code, signal) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(commandError(command, args, code, signal));
    });
  });
}

export function runCommandSync(command, args, options = {}) {
  const result = crossSpawn.sync(command, args, options);

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    throw commandError(command, args, result.status, result.signal);
  }
}

/**
 * Sends `signal` to a child started with `detached: true` and everything it started. On Windows,
 * `SIGKILL` kills the whole tree and other signals only reach the child.
 */
export function signalProcessTree(child, signal) {
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
      console.error(`failed to send ${signal} to process ${child.pid}:`, error);
    }
  }
}
