import { defineConfig } from 'tsup';

export default defineConfig({
  entry: {
    index: 'src/index.ts',
    'runtime-bridge': 'src/runtime-bridge-entry.ts',
  },
  format: ['esm'],
  dts: false,
  splitting: false,
  outDir: 'dist',
  clean: true,
  noExternal: ['sage-app-sdk'],
});
