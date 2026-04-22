import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const appRoot = resolve(import.meta.dirname, '..');
const stylesDir = resolve(appRoot, 'src/styles');
const srcDir = resolve(appRoot, 'src');
const requiredFiles = [
  resolve(stylesDir, 'tokens.css'),
  resolve(stylesDir, 'reference-tokens.css'),
  resolve(stylesDir, 'semantic-tokens.css'),
  resolve(stylesDir, 'component-tokens.css'),
  resolve(stylesDir, 'ds-tokens.css'),
];

const missingFiles = requiredFiles.filter((file) => !existsSync(file));
if (missingFiles.length > 0) {
  console.error('Design-system guard failed: missing required token files.');
  for (const file of missingFiles) {
    console.error(`- ${file}`);
  }
  process.exit(1);
}

const expectedTokensEntry = [
  "@import './reference-tokens.css';",
  "@import './semantic-tokens.css';",
  "@import './component-tokens.css';",
].join('\n');

const actualTokensEntry = readFileSync(resolve(stylesDir, 'tokens.css'), 'utf8').trim();
if (actualTokensEntry !== expectedTokensEntry) {
  console.error(
    'Design-system guard failed: src/styles/tokens.css must only re-export the three token layers in order.',
  );
  process.exit(1);
}

for (const fileName of ['reference-tokens.css', 'semantic-tokens.css', 'component-tokens.css']) {
  const content = readFileSync(resolve(stylesDir, fileName), 'utf8');
  if (!content.includes(':root')) {
    console.error(`Design-system guard failed: ${fileName} must define tokens in :root.`);
    process.exit(1);
  }
}

function collectSourceFiles(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      return collectSourceFiles(path);
    }
    if (statSync(path).isFile() && /\.(tsx|ts|css)$/.test(entry.name)) {
      return [path];
    }
    return [];
  });
}

const forbiddenRadiusPattern = /rounded-\[(24px|28px)\]/;
const filesWithHardcodedSurfaceRadius = collectSourceFiles(srcDir).filter((file) =>
  forbiddenRadiusPattern.test(readFileSync(file, 'utf8')),
);

if (filesWithHardcodedSurfaceRadius.length > 0) {
  console.error(
    'Design-system guard failed: use --radius-card / --radius-hero tokens instead of hardcoded 24px/28px rounded utilities.',
  );
  for (const file of filesWithHardcodedSurfaceRadius) {
    console.error(`- ${file}`);
  }
  process.exit(1);
}

console.log('Design-system guard passed.');
