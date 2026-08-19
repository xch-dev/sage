import { cpSync, existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');
const source = join(repoRoot, 'builtin-apps/build/dist');
const destination = join(repoRoot, 'src-tauri/gen/apple/assets/builtin-apps');

if (!existsSync(source)) {
  throw new Error(`missing built-in app resources at ${source}`);
}

rmSync(destination, { recursive: true, force: true });
mkdirSync(dirname(destination), { recursive: true });
cpSync(source, destination, { recursive: true });

console.log('Synchronized fresh built-in app resources for iOS.');
