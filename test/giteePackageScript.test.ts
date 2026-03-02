import assert from "node:assert/strict";
import test from "node:test";

import {
  chooseArchiveName,
  shouldExcludeFromSource,
} from "../scripts/prepare-gitee-package.mjs";

test("chooseArchiveName prioritizes explicit override", () => {
  const name = chooseArchiveName(["MySkills-Manager-v3.1.0-source.zip"], "0.1.3", "custom.zip");
  assert.equal(name, "custom.zip");
});

test("chooseArchiveName reuses existing gitee source archive name", () => {
  const name = chooseArchiveName(["MySkills-Manager-v3.1.0-source.zip"], "0.1.3");
  assert.equal(name, "MySkills-Manager-v3.1.0-source.zip");
});

test("chooseArchiveName falls back to package version when no existing archive", () => {
  const name = chooseArchiveName([], "0.1.3");
  assert.equal(name, "MySkills-Manager-v0.1.3-source.zip");
});

test("shouldExcludeFromSource filters build and dependency outputs", () => {
  assert.equal(shouldExcludeFromSource("node_modules/a.txt"), true);
  assert.equal(shouldExcludeFromSource("dist/assets/index.js"), true);
  assert.equal(shouldExcludeFromSource("release/Skillar.exe"), true);
  assert.equal(shouldExcludeFromSource("src-tauri/target/release/app.exe"), true);
  assert.equal(shouldExcludeFromSource(".git/config"), true);
  assert.equal(shouldExcludeFromSource("src/main.tsx"), false);
  assert.equal(shouldExcludeFromSource("scripts/expose-skillar-exe.mjs"), false);
});
