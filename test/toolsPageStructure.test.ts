import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

function readToolsPageSource() {
  const pagePath = path.resolve(process.cwd(), "src/pages/ToolsPage.tsx");
  return fs.readFileSync(pagePath, "utf8");
}

test("ToolsPage delegates tool card and custom form rendering to subcomponents", () => {
  const source = readToolsPageSource();

  assert.ok(
    !source.includes("function renderToolCard("),
    "Tool card rendering should be extracted from ToolsPage",
  );
  assert.ok(
    !source.includes("chart-card tools-form-card"),
    "Custom tool form markup should be extracted from ToolsPage",
  );
  assert.ok(
    source.includes("<ToolCard"),
    "ToolsPage should render ToolCard component",
  );
  assert.ok(
    source.includes("<CustomToolFormCard"),
    "ToolsPage should render CustomToolFormCard component",
  );
});
