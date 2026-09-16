import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { execFileSync, spawnSync } from 'node:child_process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';
import { bech32m } from 'bech32';

const cliPath = fileURLToPath(
  new URL('../cli/finalize-manifest.mjs', import.meta.url),
);

function createFixture(sourceOverrides = {}) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'sage-app-manifest-'));
  const dist = path.join(root, 'dist');
  const source = path.join(root, 'sage-manifest.source.json');

  fs.mkdirSync(dist);
  fs.writeFileSync(
    source,
    JSON.stringify({
      name: 'Test App',
      version: '1.0.0',
      sageVersion: { min: '0.13.0' },
      ...sourceOverrides,
    }),
  );

  return { root, dist, source, out: path.join(dist, 'sage-manifest.json') };
}

function finalize(fixture, extraArgs = []) {
  execFileSync(process.execPath, [
    cliPath,
    'finalize-manifest',
    '--source',
    fixture.source,
    '--dist',
    fixture.dist,
    ...extraArgs,
  ]);

  return JSON.parse(fs.readFileSync(fixture.out, 'utf8'));
}

function createDonationFixture(address) {
  const fixture = createFixture({ donation: { address } });
  fs.writeFileSync(path.join(fixture.dist, 'index.html'), '<!doctype html>');
  return fixture;
}

function finalizeResult(fixture) {
  return spawnSync(
    process.execPath,
    [
      cliPath,
      'finalize-manifest',
      '--source',
      fixture.source,
      '--dist',
      fixture.dist,
    ],
    { encoding: 'utf8' },
  );
}

test('does not exclude files by default', (t) => {
  const fixture = createFixture();
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  fs.writeFileSync(path.join(fixture.dist, 'index.html'), '<h1>Hello</h1>');
  fs.writeFileSync(
    path.join(fixture.dist, '_headers'),
    '/*\n  X-Frame-Options: DENY',
  );
  fs.writeFileSync(path.join(fixture.dist, '_redirects'), '/* /index.html 200');

  const manifest = finalize(fixture);

  assert.deepEqual(
    manifest.files.map((file) => file.path),
    ['_headers', '_redirects', 'index.html'],
  );
});

test('applies custom exclude globs and omits them from the final manifest', (t) => {
  const fixture = createFixture();
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  fs.mkdirSync(path.join(fixture.dist, 'assets'));
  fs.mkdirSync(path.join(fixture.dist, 'deployment'));
  fs.writeFileSync(path.join(fixture.dist, 'index.html'), '<h1>Hello</h1>');
  fs.writeFileSync(
    path.join(fixture.dist, '_headers'),
    '/*\n  X-Frame-Options: DENY',
  );
  fs.writeFileSync(path.join(fixture.dist, '_redirects'), '/* /index.html 200');
  fs.writeFileSync(path.join(fixture.dist, 'app.js.map'), '{}');
  fs.writeFileSync(path.join(fixture.dist, 'assets', 'vendor.js.map'), '{}');
  fs.writeFileSync(
    path.join(fixture.dist, 'deployment', 'metadata.json'),
    '{}',
  );

  const manifest = finalize(fixture, [
    '--exclude',
    '_headers',
    '--exclude',
    '_redirects',
    '--exclude',
    '**/*.map',
    '--exclude',
    'deployment/**',
  ]);

  assert.equal('exclude' in manifest, false);
  assert.deepEqual(
    manifest.files.map((file) => file.path),
    ['index.html'],
  );
  assert.equal(manifest.files[0].size, 14);
  assert.equal(
    manifest.files[0].sha256,
    crypto.createHash('sha256').update('<h1>Hello</h1>').digest('hex'),
  );
});

test('rejects source-manifest exclude configuration', (t) => {
  const fixture = createFixture({ exclude: ['**/*.map'] });
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(fixture.dist, 'index.html'), '<h1>Hello</h1>');

  const result = spawnSync(
    process.execPath,
    [
      cliPath,
      'finalize-manifest',
      '--source',
      fixture.source,
      '--dist',
      fixture.dist,
    ],
    { encoding: 'utf8' },
  );

  assert.equal(result.status, 1);
  assert.match(
    result.stderr,
    /Source manifest exclude is not supported; use --exclude <glob>/,
  );
});

test('rejects --exclude without a glob', (t) => {
  const fixture = createFixture();
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));
  fs.writeFileSync(path.join(fixture.dist, 'index.html'), '<h1>Hello</h1>');

  const result = spawnSync(
    process.execPath,
    [
      cliPath,
      'finalize-manifest',
      '--source',
      fixture.source,
      '--dist',
      fixture.dist,
      '--exclude',
    ],
    { encoding: 'utf8' },
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /Missing value for --exclude/);
});

test('accepts a valid XCH donation address', (t) => {
  const address =
    'xch10hwpheqwv4m4dzuqc5hp8se3p0chkrkvmtf8m6d830wfagg9p22sxd82v7';
  const fixture = createDonationFixture(address);
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  const manifest = finalize(fixture);

  assert.equal(manifest.donation.address, address);
});

test('rejects a valid TXCH donation address', (t) => {
  const fixture = createDonationFixture(
    'txch19hutewzq3z4l6y3fsw5laatre79tuz5p43jlvag0yz466xx9l7vs4vnpem',
  );
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  const result = finalizeResult(fixture);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid xch address/);
  assert.equal(fs.existsSync(fixture.out), false);
});

test('rejects a donation address with an invalid checksum', (t) => {
  const fixture = createDonationFixture(
    'xch10hwpheqwv4m4dzuqc5hp8se3p0chkrkvmtf8m6d830wfagg9p22sxd82v8',
  );
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  const result = finalizeResult(fixture);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid xch address/);
  assert.equal(fs.existsSync(fixture.out), false);
});

test('rejects a valid Bech32m address with a non-wallet prefix', (t) => {
  const address = bech32m.encode('nft', bech32m.toWords(Buffer.alloc(32, 1)));
  const fixture = createDonationFixture(address);
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  const result = finalizeResult(fixture);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid xch address/);
});

test('rejects a valid XCH address with the wrong payload length', (t) => {
  const address = bech32m.encode('xch', bech32m.toWords(Buffer.alloc(31, 1)));
  const fixture = createDonationFixture(address);
  t.after(() => fs.rmSync(fixture.root, { recursive: true, force: true }));

  const result = finalizeResult(fixture);

  assert.equal(result.status, 1);
  assert.match(result.stderr, /donation\.address must be a valid xch address/);
});
