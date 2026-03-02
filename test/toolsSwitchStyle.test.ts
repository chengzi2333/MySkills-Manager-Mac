import assert from "node:assert/strict";
import test from "node:test";
import fs from "node:fs";
import path from "node:path";

test("tool switch active state uses common green/white palette", () => {
  const cssPath = path.resolve("src/pages/ToolsPage.css");
  const css = fs.readFileSync(cssPath, "utf8");

  assert.match(
    css,
    /\.tool-switch\.active\s*\{[\s\S]*background:\s*#22c55e;[\s\S]*\}/,
  );

  assert.match(
    css,
    /\.tool-switch-thumb\s*\{[\s\S]*background:\s*#ffffff;[\s\S]*\}/,
  );
});
