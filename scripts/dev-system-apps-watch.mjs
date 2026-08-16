import chokidar from 'chokidar';
import { WebSocketServer } from 'ws';
import { runCommand } from './run-command.mjs';

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
let scheduledApps = new Set();

function broadcast(payload) {
  const text = JSON.stringify(payload);

  for (const client of wss.clients) {
    if (client.readyState === client.OPEN) {
      client.send(text);
    }
  }
}

function systemAppNameFromPath(path) {
  const match = path.match(/^builtin-apps\/src\/system\/apps\/([^/]+)\//);
  return match?.[1] ?? null;
}

async function rebuild({ packages = false, apps = [] } = {}) {
  if (running) {
    queuedSystemApps = true;
    queuedPackages ||= packages;

    for (const app of apps) {
      scheduledApps.add(app);
    }

    return;
  }

  running = true;

  try {
    if (packages) {
      console.log('\n[system-apps-dev] rebuilding shared packages...');
      await runCommand('pnpm', ['run', 'build:packages'], {
        stdio: 'inherit',
      });
    }

    const buildArgs =
      apps.length > 0
        ? ['run', 'build:system-apps', '--', ...apps]
        : ['run', 'build:system-apps'];

    console.log(
      apps.length > 0
        ? `\n[system-apps-dev] rebuilding system apps: ${apps.join(', ')}`
        : '\n[system-apps-dev] rebuilding all system apps...',
    );

    await runCommand('pnpm', buildArgs, { stdio: 'inherit' });

    broadcast({
      type: 'system-apps-built',
      ok: true,
      apps,
      at: Date.now(),
    });

    console.log('\n[system-apps-dev] done');
  } catch (err) {
    console.error('\n[system-apps-dev] build failed:', err);

    broadcast({
      type: 'system-apps-built',
      ok: false,
      error: err instanceof Error ? err.message : String(err),
      apps,
      at: Date.now(),
    });
  } finally {
    running = false;

    if (queuedSystemApps || queuedPackages || scheduledApps.size > 0) {
      const nextPackages = queuedPackages;
      const nextApps = [...scheduledApps];

      queuedSystemApps = false;
      queuedPackages = false;
      scheduledApps = new Set();

      void rebuild({
        packages: nextPackages,
        apps: nextPackages ? [] : nextApps,
      });
    }
  }
}

function schedule({ packages = false, appName = null } = {}) {
  scheduledNeedsPackages ||= packages;

  if (appName) {
    scheduledApps.add(appName);
  }

  if (timer) {
    clearTimeout(timer);
  }

  timer = setTimeout(() => {
    const packages = scheduledNeedsPackages;
    const apps = [...scheduledApps];

    timer = null;
    scheduledNeedsPackages = false;
    scheduledApps = new Set();

    void rebuild({
      packages,
      apps: packages ? [] : apps,
    });
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
    const appName = packages ? null : systemAppNameFromPath(path);

    console.log(
      `[system-apps-dev] ${event}: ${path}${
        packages ? ' [packages]' : appName ? ` [${appName}]` : ' [all]'
      }`,
    );

    schedule({
      packages,
      appName,
    });
  });
