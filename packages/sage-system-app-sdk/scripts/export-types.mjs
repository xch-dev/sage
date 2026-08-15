import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import ts from 'typescript';

const packageDir = process.cwd();
const repoRoot = path.resolve(packageDir, '../..');

const systemOutPath = path.join(packageDir, 'src', 'generated-types.ts');
const userTypesPath = path.join(
  repoRoot,
  'packages',
  'sage-app-sdk',
  'src',
  'generated-types.ts',
);

const systemSource = execFileSync(
  'cargo',
  ['run', '-p', 'sage-apps', '--bin', 'export_bridge_types', '--', 'system'],
  {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'inherit'],
  },
);

const userSource = fs.existsSync(userTypesPath)
  ? fs.readFileSync(userTypesPath, 'utf8')
  : '';

function collectExportedNames(source, fileName) {
  const file = ts.createSourceFile(
    fileName,
    source,
    ts.ScriptTarget.Latest,
    true,
  );

  const names = new Set();

  for (const node of file.statements) {
    const modifiers = ts.getModifiers(node) ?? [];
    const exported = modifiers.some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    );

    if (!exported) continue;

    if (
      (ts.isTypeAliasDeclaration(node) ||
        ts.isInterfaceDeclaration(node) ||
        ts.isEnumDeclaration(node) ||
        ts.isClassDeclaration(node) ||
        ts.isFunctionDeclaration(node)) &&
      node.name
    ) {
      names.add(node.name.text);
    }
  }

  return names;
}

function stripDuplicateExportedDeclarations(source, fileName, duplicateNames) {
  const file = ts.createSourceFile(
    fileName,
    source,
    ts.ScriptTarget.Latest,
    true,
  );

  const ranges = [];

  for (const node of file.statements) {
    const modifiers = ts.getModifiers(node) ?? [];
    const exported = modifiers.some(
      (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
    );

    if (!exported) continue;

    const name =
      (ts.isTypeAliasDeclaration(node) ||
        ts.isInterfaceDeclaration(node) ||
        ts.isEnumDeclaration(node) ||
        ts.isClassDeclaration(node) ||
        ts.isFunctionDeclaration(node)) &&
      node.name
        ? node.name.text
        : null;

    if (!name || !duplicateNames.has(name)) continue;

    ranges.push([node.getFullStart(), node.getEnd()]);
  }

  let next = source;

  for (const [start, end] of ranges.sort((a, b) => b[0] - a[0])) {
    next = next.slice(0, start) + next.slice(end);
  }

  return next.replace(/\n{3,}/g, '\n\n').trimStart();
}

function collectReferencedDuplicateNames(source, duplicateNames) {
  const referenced = [];

  for (const name of duplicateNames) {
    const regex = new RegExp(`\\b${name}\\b`, 'g');

    if (regex.test(source)) {
      referenced.push(name);
    }
  }

  return referenced.sort();
}

function prependUserSdkTypeImports(source, names) {
  if (names.length === 0) {
    return source;
  }

  return `import type {\n${names
    .map((name) => `  ${name},`)
    .join('\n')}\n} from '@sage-app/sdk';\n\n${source}`;
}

const userNames = collectExportedNames(userSource, userTypesPath);

let cleanedSystemSource = stripDuplicateExportedDeclarations(
  systemSource,
  'system-generated-types.ts',
  userNames,
);

const importsFromUserSdk = collectReferencedDuplicateNames(
  cleanedSystemSource,
  userNames,
);

cleanedSystemSource = prependUserSdkTypeImports(
  cleanedSystemSource,
  importsFromUserSdk,
);

fs.writeFileSync(systemOutPath, cleanedSystemSource);
console.log(`Wrote ${path.relative(packageDir, systemOutPath)}`);
