import { lingui } from '@lingui/vite-plugin';
import react from '@vitejs/plugin-react-swc';
import path from 'path';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react({
      plugins: [['@lingui/swc-plugin', {}]],
    }),
    lingui(),
  ],
  clearScreen: false,
  server: {
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
