import { mkdir, copyFile, readdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, "..");

const dmgDir = path.join(
  projectRoot,
  "src-tauri",
  "target",
  "release",
  "bundle",
  "dmg",
);
const releaseDir = path.join(projectRoot, "release");

let dmgFile;
try {
  const files = await readdir(dmgDir);
  dmgFile = files.find((f) => f.endsWith(".dmg"));
} catch {
  console.error(
    "Missing src-tauri/target/release/bundle/dmg/. Run `cargo tauri build` first.",
  );
  process.exit(1);
}

if (!dmgFile) {
  console.error("No .dmg file found in bundle/dmg/ directory.");
  process.exit(1);
}

const sourceDmg = path.join(dmgDir, dmgFile);
const targetDmg = path.join(releaseDir, "Skillar.dmg");

await mkdir(releaseDir, { recursive: true });
await copyFile(sourceDmg, targetDmg);
console.log(`Prepared DMG installer: ${targetDmg}`);
