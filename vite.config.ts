import { lingui } from '@lingui/vite-plugin';
import react from '@vitejs/plugin-react-swc';
import { readFileSync } from 'fs';
import path from 'path';
import { defineConfig, type Plugin } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// Set by `pnpm sandbox` (scripts/sandbox.mjs), which runs the UI in a browser. See docs/sandbox.md.
const sandboxBridge = process.env.SAGE_SANDBOX_BRIDGE;
const sandboxPort = Number(process.env.SAGE_SANDBOX_VITE_PORT) || undefined;

/** Loads the Tauri IPC stand-in before the app, so that it never ships outside the sandbox. */
function sandboxPlugin(): Plugin {
  return {
    name: 'sage-sandbox',
    apply: 'serve',
    transformIndexHtml: () => [
      {
        tag: 'script',
        attrs: { type: 'module', src: '/src/sandbox/install.ts' },
        injectTo: 'head-prepend',
      },
    ],
  };
}

function appVersion(): string {
  const config = JSON.parse(
    readFileSync(path.resolve(__dirname, 'src-tauri/tauri.conf.json'), 'utf8'),
  );
  return config.version;
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      plugins: [['@lingui/swc-plugin', {}]],
    }),
    lingui(),
    ...(sandboxBridge ? [sandboxPlugin()] : []),
  ],
  define: sandboxBridge
    ? { __SAGE_SANDBOX_VERSION__: JSON.stringify(appVersion()) }
    : undefined,
  // The sandbox doesn't build the mini-apps, so keep Vite from scanning their entry points.
  optimizeDeps: sandboxBridge ? { entries: ['index.html'] } : undefined,
  clearScreen: false,
  server: sandboxBridge
    ? {
        port: sandboxPort,
        strictPort: true,
        host: '127.0.0.1',
        proxy: {
          '/__sandbox': {
            target: sandboxBridge,
            rewrite: (path) => path.replace(/^\/__sandbox/, ''),
          },
        },
        watch: {
          ignored: ['**/src-tauri/**', '**/.sandbox/**', '**/target/**'],
        },
      }
    : {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
          ? {
              protocol: 'ws',
              host,
              port: 1421,
            }
          : undefined,
        watch: {
          ignored: ['**/src-tauri/**'],
        },
      },
  build: {
    chunkSizeWarningLimit: 2048,
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
