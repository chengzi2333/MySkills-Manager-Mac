import { mkdir, copyFile, access } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, "..");

const sourceExe = path.join(
  projectRoot,
  "src-tauri",
  "target",
  "release",
  "app.exe",
);
const releaseDir = path.join(projectRoot, "release");
const targetExe = path.join(releaseDir, "Skillar.exe");

try {
  await access(sourceExe);
} catch {
  console.error(
    "Missing src-tauri/target/release/app.exe. Run `cargo tauri build` first.",
  );
  process.exit(1);
}

await mkdir(releaseDir, { recursive: true });
await copyFile(sourceExe, targetExe);
console.log(`Prepared launcher: ${targetExe}`);
