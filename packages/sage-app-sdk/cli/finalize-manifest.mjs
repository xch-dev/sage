#!/usr/bin/env node

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import picomatch from 'picomatch';
import { bech32m } from 'bech32';

const MAX_MANIFEST_SIZE_BYTES = 1024 * 1024;
const MAX_FILE_COUNT = 2000;
const MAX_TOTAL_SIZE_BYTES = 50 * 1024 * 1024;

function fail(message) {
  console.error(`Error: ${message}`);
  process.exit(1);
}

function printHelp() {
  console.log(
    `
Usage:
  sage-app finalize-manifest --source ./sage-manifest.json --dist ./dist [--out ./dist/sage-manifest.json] [--exclude <glob>]...

Commands:
  finalize-manifest   Generate final sage-manifest.json from source manifest and built dist files

Options:
  --exclude <glob>    Exclude matching dist files; repeat for multiple globs
`.trim(),
  );
}

function parseArgs(argv) {
  if (argv.length === 0) {
    printHelp();
    process.exit(1);
  }

  const command = argv[0];
  const rest = argv.slice(1);

  if (command !== 'finalize-manifest') {
    fail(`Unknown command: ${command}`);
  }

  const args = {
    source: null,
    dist: null,
    out: null,
    exclude: [],
  };

  for (let i = 0; i < rest.length; i += 1) {
    const arg = rest[i];

    if (arg === '--source') {
      args.source = rest[++i] ?? null;
    } else if (arg === '--dist') {
      args.dist = rest[++i] ?? null;
    } else if (arg === '--out') {
      args.out = rest[++i] ?? null;
    } else if (arg === '--exclude') {
      const pattern = rest[++i] ?? null;
      if (!pattern) {
        fail('Missing value for --exclude');
      }
      args.exclude.push(pattern);
    } else if (arg === '--help' || arg === '-h') {
      printHelp();
      process.exit(0);
    } else {
      fail(`Unknown argument: ${arg}`);
    }
  }

  if (!args.source) {
    fail('Missing --source');
  }

  if (!args.dist) {
    fail('Missing --dist');
  }

  if (!args.out) {
    args.out = path.join(args.dist, 'sage-manifest.json');
  }

  return args;
}

function sha256File(filePath) {
  const hash = crypto.createHash('sha256');
  const data = fs.readFileSync(filePath);
  hash.update(data);
  return hash.digest('hex');
}

function walkFiles(rootDir, isExcluded, currentDir = rootDir) {
  const out = [];
  const entries = fs.readdirSync(currentDir, { withFileTypes: true });

  for (const entry of entries) {
    const abs = path.join(currentDir, entry.name);

    if (entry.isDirectory()) {
      out.push(...walkFiles(rootDir, isExcluded, abs));
      continue;
    }

    if (!entry.isFile()) {
      continue;
    }

    const rel = path.relative(rootDir, abs).split(path.sep).join('/');

    if (
      rel === 'manifest.json' ||
      rel === 'sage-manifest.json' ||
      isExcluded(rel)
    ) {
      continue;
    }

    out.push({
      abs,
      rel,
    });
  }

  return out;
}

function validateSourceManifest(manifest) {
  if (!manifest || typeof manifest !== 'object' || Array.isArray(manifest)) {
    fail('Source manifest must be an object');
  }

  if (typeof manifest.name !== 'string' || manifest.name.trim() === '') {
    fail('Source manifest name must be a non-empty string');
  }

  if (typeof manifest.version !== 'string' || manifest.version.trim() === '') {
    fail('Source manifest version must be a non-empty string');
  }

  if (
    manifest.permissions != null &&
    (typeof manifest.permissions !== 'object' ||
      Array.isArray(manifest.permissions))
  ) {
    fail('Source manifest permissions must be an object if provided');
  }

  if ('exclude' in manifest) {
    fail('Source manifest exclude is not supported; use --exclude <glob>');
  }

  if (manifest.donation != null) {
    const address = manifest.donation?.address;

    if (typeof address !== 'string' || !isValidDonationAddress(address)) {
      fail('Source manifest donation.address must be a valid xch address');
    }
  }
}

function isValidDonationAddress(address) {
  try {
    const decoded = bech32m.decode(address);

    if (decoded.prefix !== 'xch') {
      return false;
    }

    return bech32m.fromWords(decoded.words).length === 32;
  } catch {
    return false;
  }
}

function main() {
  const args = parseArgs(process.argv.slice(2));

  const sourcePath = path.resolve(args.source);
  const distDir = path.resolve(args.dist);
  const outPath = path.resolve(args.out);

  if (!fs.existsSync(sourcePath)) {
    fail(`Source manifest not found: ${sourcePath}`);
  }

  if (!fs.existsSync(distDir) || !fs.statSync(distDir).isDirectory()) {
    fail(`Dist directory not found: ${distDir}`);
  }

  const sourceStat = fs.statSync(sourcePath);
  if (sourceStat.size > MAX_MANIFEST_SIZE_BYTES) {
    fail(`Source manifest exceeds ${MAX_MANIFEST_SIZE_BYTES} bytes`);
  }

  const sourceManifest = JSON.parse(fs.readFileSync(sourcePath, 'utf8'));
  validateSourceManifest(sourceManifest);

  const isExcluded = picomatch(args.exclude, { dot: true });
  const walked = walkFiles(distDir, isExcluded).sort((a, b) =>
    a.rel.localeCompare(b.rel),
  );

  if (walked.length === 0) {
    fail(`No files found in dist directory: ${distDir}`);
  }

  if (walked.length > MAX_FILE_COUNT) {
    fail(`File count ${walked.length} exceeds limit of ${MAX_FILE_COUNT}`);
  }

  let totalSize = 0;

  const files = walked.map(({ abs, rel }) => {
    const stat = fs.statSync(abs);
    const size = stat.size;
    totalSize += size;

    return {
      path: rel,
      sha256: sha256File(abs),
      size,
    };
  });

  if (totalSize > MAX_TOTAL_SIZE_BYTES) {
    fail(
      `Total snapshot size ${totalSize} exceeds limit of ${MAX_TOTAL_SIZE_BYTES} bytes`,
    );
  }

  const finalManifest = {
    ...sourceManifest,
    permissions: sourceManifest.permissions ?? {},
    files,
  };

  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(finalManifest, null, 2) + '\n');

  console.log(`Wrote ${outPath}`);
  console.log(`Included ${files.length} files`);
  console.log(`Total size ${totalSize} bytes`);
}

main();
