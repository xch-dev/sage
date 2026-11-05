import { copyFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const dist = resolve(root, 'dist');
mkdirSync(dist, { recursive: true });
copyFileSync(resolve(root, 'src/theme/theme.css'), resolve(dist, 'theme.css'));
copyFileSync(resolve(root, 'src/theme/theme-css.d.ts'), resolve(dist, 'theme-css.d.ts'));
