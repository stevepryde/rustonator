import { mkdir, rm, writeFile } from "node:fs/promises";
import path from "node:path";

const distDir = path.resolve("dist");
const headersPath = path.join(distDir, "_headers");
const frameAncestors = (process.env.CSP_FRAME_ANCESTORS || "").trim();

if (!frameAncestors) {
  await rm(headersPath, { force: true });
  process.exit(0);
}

await mkdir(distDir, { recursive: true });
await writeFile(
  headersPath,
  `/*\n  Content-Security-Policy: frame-ancestors ${frameAncestors}\n`,
  "utf8"
);
