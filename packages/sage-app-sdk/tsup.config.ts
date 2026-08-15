import { defineConfig } from 'tsup';

export default defineConfig([
  {
    entry: {
      index: 'src/index.ts',
    },
    format: ['esm'],
    dts: true,
    splitting: false,
    outDir: 'dist',
    clean: true,
  },
  {
    entry: {
      'runtime-bridge': 'src/runtime-bridge-entry.ts',
    },
    format: ['esm'],
    dts: false,
    splitting: false,
    outDir: 'dist',
    clean: false,
  },
]);
