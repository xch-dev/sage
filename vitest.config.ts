import { lingui } from '@lingui/vite-plugin';
import react from '@vitejs/plugin-react-swc';
import path from 'path';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [
    react({
      plugins: [['@lingui/swc-plugin', {}]],
    }),
    lingui(),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['src/__tests__/setup.ts'],
    include: ['src/**/*.test.{ts,tsx}'],
  },
});
