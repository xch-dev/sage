import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { basename, dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function defineSageSystemAppConfig(importMetaUrl: string) {
  const dir = dirname(fileURLToPath(importMetaUrl));
  const appName = basename(dir);
  const workspaceRoot = resolve(dir, '../../../../../..');

  return defineConfig({
    root: dir,
    plugins: [react()],
    publicDir: resolve(dir, 'public'),
    build: {
      outDir: resolve(
        workspaceRoot,
        'target',
        'sage-builtin-apps',
        'work',
        'system-apps',
        appName,
        'dist',
      ),
      emptyOutDir: true,
      rollupOptions: {
        input: resolve(dir, 'index.html'),
      },
    },
  });
}
