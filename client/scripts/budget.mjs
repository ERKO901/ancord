// Rozpočet JS bundle (§13.11): < 150 kB gzip. Spouští se po `npm run build`; selže při překročení.
import { readdirSync, readFileSync, existsSync } from "node:fs";
import { gzipSync } from "node:zlib";

const LIMIT_JS = 150 * 1024;
const dir = new URL("../dist/assets/", import.meta.url);
if (!existsSync(dir)) {
  console.error("Chybí dist/. Spusť nejdřív `npm run build`.");
  process.exit(1);
}
const sizes = { js: 0, css: 0 };
for (const f of readdirSync(dir)) {
  const kind = f.endsWith(".js") ? "js" : f.endsWith(".css") ? "css" : null;
  if (kind) sizes[kind] += gzipSync(readFileSync(new URL(f, dir))).length;
}
const kb = (n) => (n / 1024).toFixed(1);
console.log(`JS ${kb(sizes.js)} kB gzip (limit ${kb(LIMIT_JS)}), CSS ${kb(sizes.css)} kB gzip`);
if (sizes.js > LIMIT_JS) {
  console.error("Rozpočet JS bundle překročen.");
  process.exit(1);
}
