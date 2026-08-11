import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const dir = dirname(fileURLToPath(import.meta.url));

export default {
  plugins: {
    tailwindcss: {
      config: join(dir, 'tailwind.config.js'),
    },
    autoprefixer: {},
  },
};
