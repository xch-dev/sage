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
