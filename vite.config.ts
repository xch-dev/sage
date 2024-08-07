import react from '@vitejs/plugin-react-swc';
import { defineConfig } from 'vite';

const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    host: host || false,
    strictPort: true,
  },
});
