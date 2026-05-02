import chokidar from 'chokidar';
import { WebSocketServer } from 'ws';
import { spawn } from 'node:child_process';

const PORT = 1421;

const watchPaths = [
  'builtin-apps/src/system/apps/**/*',
  'builtin-apps/src/system/*.ts',
  'builtin-apps/src/system/*.js',
  'packages/sage-app-sdk/src/**/*',
  'packages/sage-system-app-sdk/src/**/*',
  'packages/sage-app-ui/src/**/*',
];

const ignored = [
  '**/node_modules/**',
  '**/dist/**',
  'builtin-apps/build/**',
  'packages/*/src/generated-types.ts',
];

const wss = new WebSocketServer({ port: PORT });

let timer = null;
let running = false;
let queuedPackages = false;
let queuedSystemApps = false;
let scheduledNeedsPackages = false;

function run(command, args) {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      stdio: 'inherit',
      shell: process.platform === 'win32',
    });

    child.on('exit', (code) => {
      if (code === 0) resolve();
      else
        reject(new Error(`${command} ${args.join(' ')} failed with ${code}`));
    });
  });
}

function broadcast(payload) {
  const text = JSON.stringify(payload);

  for (const client of wss.clients) {
    if (client.readyState === client.OPEN) {
      client.send(text);
    }
  }
}

async function rebuild({ packages = false } = {}) {
  if (running) {
    queuedSystemApps = true;
    queuedPackages ||= packages;
    return;
  }

  running = true;

  try {
    if (packages) {
      console.log('\n[system-apps-dev] rebuilding shared packages...');
      await run('pnpm', ['run', 'build:packages']);
    }

    console.log('\n[system-apps-dev] rebuilding system apps...');
    await run('pnpm', ['run', 'build:system-apps']);

    broadcast({
      type: 'system-apps-built',
      ok: true,
      at: Date.now(),
    });

    console.log('\n[system-apps-dev] done');
  } catch (err) {
    console.error('\n[system-apps-dev] build failed:', err);

    broadcast({
      type: 'system-apps-built',
      ok: false,
      error: err instanceof Error ? err.message : String(err),
      at: Date.now(),
    });
  } finally {
    running = false;

    if (queuedSystemApps || queuedPackages) {
      const nextPackages = queuedPackages;
      queuedSystemApps = false;
      queuedPackages = false;

      void rebuild({ packages: nextPackages });
    }
  }
}

function schedule({ packages = false } = {}) {
  scheduledNeedsPackages ||= packages;

  if (timer) {
    clearTimeout(timer);
  }

  timer = setTimeout(() => {
    const packages = scheduledNeedsPackages;

    timer = null;
    scheduledNeedsPackages = false;

    void rebuild({ packages });
  }, 150);
}

console.log(`[system-apps-dev] websocket listening on ws://127.0.0.1:${PORT}`);
console.log('[system-apps-dev] watching system apps + SDK/UI packages');

chokidar
  .watch(watchPaths, {
    ignored,
    ignoreInitial: true,
  })
  .on('all', (event, path) => {
    const packages = path.startsWith('packages/');

    console.log(
      `[system-apps-dev] ${event}: ${path}${packages ? ' [packages]' : ''}`,
    );

    schedule({ packages });
  });
