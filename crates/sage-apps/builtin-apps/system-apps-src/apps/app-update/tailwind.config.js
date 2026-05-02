import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createSageTailwindConfig } from '../../tailwind.shared.js';

const dir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(dir, '../../../../../..');

export default createSageTailwindConfig({
  content: [
    join(dir, 'index.html'),
    join(dir, 'src/**/*.{ts,tsx}'),
    join(repoRoot, 'packages/sage-app-ui/src/**/*.{ts,tsx}'),
  ],
});
