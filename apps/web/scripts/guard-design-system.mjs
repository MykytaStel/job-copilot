import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { resolve } from 'node:path';

const appRoot = resolve(import.meta.dirname, '..');
const stylesDir = resolve(appRoot, 'src/styles');
const srcDir = resolve(appRoot, 'src');
const componentsDir = resolve(srcDir, 'components');
const pagesDir = resolve(srcDir, 'pages');

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

function collectSourceFiles(dir, extensions = /\.(tsx|ts|css)$/) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = resolve(dir, entry.name);
    if (entry.isDirectory()) {
      return collectSourceFiles(path, extensions);
    }
    if (statSync(path).isFile() && extensions.test(entry.name)) {
      return [path];
    }
    return [];
  });
}

const errors = [];

// ── Rule 1: no hardcoded 24px/28px rounded utilities ─────────────────────────
const forbiddenRadiusPattern = /rounded-\[(24px|28px)\]/;
for (const file of collectSourceFiles(srcDir)) {
  if (forbiddenRadiusPattern.test(readFileSync(file, 'utf8'))) {
    errors.push(
      `Use --radius-card / --radius-hero tokens instead of hardcoded 24px/28px rounded: ${file}`,
    );
  }
}

// ── Rule 2: no raw bg-white/[0.0x] opacity classes in TSX/TS ─────────────────
const rawOpacityPattern = /bg-white\/\[0\.\d+\]/;
for (const file of collectSourceFiles(srcDir, /\.(tsx|ts)$/)) {
  if (rawOpacityPattern.test(readFileSync(file, 'utf8'))) {
    errors.push(
      `Use semantic token classes (bg-surface-muted, bg-white-a04, etc.) instead of raw bg-white/[0.x]: ${file}`,
    );
  }
}

// ── Rule 3: no rgba() literals in component/page TSX files ───────────────────
// Allows rgba() in CSS token files only.
const rgbaPattern = /\brgba\s*\(/;
for (const file of [...collectSourceFiles(componentsDir, /\.(tsx|ts)$/), ...collectSourceFiles(pagesDir, /\.(tsx|ts)$/)]) {
  if (rgbaPattern.test(readFileSync(file, 'utf8'))) {
    errors.push(
      `Use CSS variable tokens instead of raw rgba() in component/page files: ${file}`,
    );
  }
}

// ── Rule 4: no hardcoded 24px/28px border-radius in CSS outside token files ──
// Only blocks the specific values that have semantic token equivalents.
const hardcodedBorderRadiusCssPattern = /border-radius\s*:\s*(24px|28px)/;
const tokenFileNames = new Set(requiredFiles.map((f) => f));
for (const file of collectSourceFiles(srcDir, /\.css$/)) {
  if (tokenFileNames.has(file)) continue;
  if (hardcodedBorderRadiusCssPattern.test(readFileSync(file, 'utf8'))) {
    errors.push(
      `Use var(--radius-card) or var(--radius-hero) instead of hardcoded 24px/28px border-radius: ${file}`,
    );
  }
}

if (errors.length > 0) {
  console.error('Design-system guard failed:');
  for (const err of errors) {
    console.error(`  ✗ ${err}`);
  }
  process.exit(1);
}

console.log('Design-system guard passed.');
