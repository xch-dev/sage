import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

export function defineSageSystemAppConfig(importMetaUrl: string) {
  const dir = dirname(fileURLToPath(importMetaUrl));

  return defineConfig({
    root: dir,
    plugins: [react()],
    publicDir: resolve(dir, 'public'),
    build: {
      outDir: resolve(dir, 'dist'),
      emptyOutDir: true,
      rollupOptions: {
        input: resolve(dir, 'index.html'),
      },
    },
  });
}
