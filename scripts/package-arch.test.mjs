import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import test from 'node:test';

test('Arch uses the working-tree version and orders release candidates before releases', () => {
  const checkout = mkdtempSync(join(tmpdir(), 'sage-arch-version-'));
  const archDir = join(checkout, 'src-tauri/arch');
  try {
    mkdirSync(archDir, { recursive: true });
    copyFileSync(
      resolve(import.meta.dirname, '../src-tauri/arch/PKGBUILD'),
      join(archDir, 'PKGBUILD'),
    );
    for (const [version, expected] of [
      ['0.13.0', '0.13.0'],
      ['0.14.0-rc.1', '0.14.0rc.1'],
      ['0.14.0', '0.14.0'],
    ]) {
      writeFileSync(
        join(checkout, 'src-tauri/tauri.conf.json'),
        JSON.stringify({ version }),
      );
      assert.equal(
        execFileSync(
          'bash',
          ['-c', 'source "$1/PKGBUILD"; pkgver', 'test', archDir],
          { encoding: 'utf8' },
        ).trim(),
        expected,
      );
    }
    assert.equal(
      execFileSync('vercmp', ['0.14.0rc.1', '0.14.0'], {
        encoding: 'utf8',
      }).trim(),
      '-1',
    );
  } finally {
    rmSync(checkout, { recursive: true, force: true });
  }
});

test('Arch launcher preserves arguments and allows overriding the Wayland default', () => {
  const launcher = readFileSync(
    resolve(import.meta.dirname, '../src-tauri/arch/sage-tauri'),
    'utf8',
  );
  const command = 'exec /usr/lib/sage-wallet/sage-tauri "$@"';
  assert.ok(
    launcher.includes(command),
    'Launcher must forward arguments to the packaged binary',
  );
  // Substitute the final exec to observe the real launcher's environment without opening a wallet.
  const probe = launcher.replace(
    command,
    'printf "%s\\n" "${__NV_DISABLE_EXPLICIT_SYNC-unset}" "$@"',
  );
  const env = { ...process.env };
  delete env.WAYLAND_DISPLAY;
  delete env.__NV_DISABLE_EXPLICIT_SYNC;
  for (const [overrides, expected] of [
    [{}, 'unset'],
    [{ WAYLAND_DISPLAY: 'wayland-0' }, '1'],
    [{ WAYLAND_DISPLAY: 'wayland-0', __NV_DISABLE_EXPLICIT_SYNC: '0' }, '0'],
  ]) {
    assert.equal(
      execFileSync(
        'sh',
        ['-c', probe, 'sage-tauri', 'argument with spaces', '--flag'],
        {
          env: { ...env, ...overrides },
          encoding: 'utf8',
        },
      ),
      `${expected}\nargument with spaces\n--flag\n`,
    );
  }
});
