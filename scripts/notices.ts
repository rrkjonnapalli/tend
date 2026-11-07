// Writes static/third-party-licenses.txt: the licence notice of every library and font Tend ships
// (MIT, BSD, Apache and OFL all ask for their notice to go along). Settings → About shows it.
// Run with `bun run notices` after changing dependencies; the file is committed.
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';

type Shipped = { name: string; version: string; license: string; dir: string };

const ROOT = join(import.meta.dirname, '..');
const OUT = join(ROOT, 'static', 'third-party-licenses.txt');
/** Licence files, best first: for "MIT OR Apache-2.0" the MIT text is the one we follow. */
const LICENSE_FILE = /^(licen[sc]e|copying|notice)/i;
const PREFERRED = /mit/i;

// ---- the web side: package.json "dependencies" and everything they pull in

function npmPackages(): Shipped[] {
  const found = new Map<string, Shipped>();
  const visit = (name: string) => {
    if (found.has(name)) return;
    const dir = join(ROOT, 'node_modules', name);
    const manifest = join(dir, 'package.json');
    if (!existsSync(manifest)) return;
    const pkg = JSON.parse(readFileSync(manifest, 'utf8'));
    found.set(name, { name, version: pkg.version, license: String(pkg.license ?? 'unknown'), dir });
    for (const dep of Object.keys(pkg.dependencies ?? {})) visit(dep);
  };
  const app = JSON.parse(readFileSync(join(ROOT, 'package.json'), 'utf8'));
  for (const dep of Object.keys(app.dependencies)) visit(dep);
  return [...found.values()];
}

// ---- the Rust side: normal (not build or dev) dependencies, for every target Tend builds

type MetaPackage = { id: string; name: string; version: string; license: string | null; manifest_path: string };
type MetaNode = { id: string; deps: { pkg: string; dep_kinds: { kind: string | null }[] }[] };

function cratePackages(): Shipped[] {
  const text = execFileSync('cargo', ['metadata', '--format-version', '1', '--locked'], {
    cwd: join(ROOT, 'src-tauri'),
    maxBuffer: 64 * 1024 * 1024,
  }).toString();
  const meta = JSON.parse(text);
  const packages = new Map<string, MetaPackage>(meta.packages.map((p: MetaPackage) => [p.id, p]));
  const nodes = new Map<string, MetaNode>(meta.resolve.nodes.map((n: MetaNode) => [n.id, n]));
  const root: string = meta.resolve.root;
  const seen = new Set<string>();
  const queue = [root];
  while (queue.length > 0) {
    const id = queue.pop()!;
    if (seen.has(id)) continue;
    seen.add(id);
    for (const dep of nodes.get(id)?.deps ?? []) {
      const normal = dep.dep_kinds.some((k) => k.kind === null);
      // A `-src` build dependency (openssl-src) holds C code compiled into the app.
      const vendored = packages.get(dep.pkg)?.name.endsWith('-src') ?? false;
      if (normal || vendored) queue.push(dep.pkg);
    }
  }
  seen.delete(root);
  return [...seen].map((id) => {
    const p = packages.get(id)!;
    return { name: p.name, version: p.version, license: p.license ?? 'unknown', dir: dirname(p.manifest_path) };
  });
}

// ---- the notice text

function noticeOf(dir: string): string | null {
  const files = readdirSync(dir).filter((f) => LICENSE_FILE.test(f));
  files.sort((a, b) => Number(PREFERRED.test(b)) - Number(PREFERRED.test(a)));
  const first = files[0];
  if (!first) return null;
  return readFileSync(join(dir, first), 'utf8').trim();
}

/** Packages with the same notice text share one copy of it. */
function grouped(all: Shipped[]): string {
  const byText = new Map<string, string[]>();
  const without: string[] = [];
  for (const p of all) {
    const label = `${p.name} ${p.version} (${p.license})`;
    const text = noticeOf(p.dir);
    if (text === null) {
      without.push(label);
      continue;
    }
    byText.set(text, [...(byText.get(text) ?? []), label]);
  }
  const rule = '-'.repeat(72);
  const sections = [...byText].map(([text, names]) => `${names.sort().join('\n')}\n\n${text}\n`);
  if (without.length > 0) {
    sections.push(`No licence file in the package; its licence is the one named:\n\n${without.sort().join('\n')}\n`);
  }
  return sections.join(`\n${rule}\n\n`);
}

const header = `Tend ships these libraries and fonts. Their licences ask for these notices to go along.

Smile Moon (static/fonts/Smile Moon.otf) is by MJType, all rights reserved.
`;
const rule = '='.repeat(72);
const body = [
  header,
  `${rule}\nThe app's interface (JavaScript libraries and fonts)\n${rule}\n\n${grouped(npmPackages())}`,
  `${rule}\nThe app itself (Rust crates)\n${rule}\n\n${grouped(cratePackages())}`,
].join('\n');
writeFileSync(OUT, body);
console.log(`wrote ${OUT} (${Math.round(body.length / 1024)} KB)`);
