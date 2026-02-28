import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

function readToolsPageSource() {
  const pagePath = path.resolve(process.cwd(), "src/pages/ToolsPage.tsx");
  return fs.readFileSync(pagePath, "utf8");
}

function readToolCardSource() {
  const cardPath = path.resolve(process.cwd(), "src/pages/tools/ToolCard.tsx");
  return fs.readFileSync(cardPath, "utf8");
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
    !source.includes("<ToolCard"),
    "ToolsPage should not render ToolCard directly",
  );
  assert.ok(
    source.includes("<CustomToolFormCard"),
    "ToolsPage should render CustomToolFormCard component",
  );
  assert.ok(
    source.includes("<ToolSection"),
    "ToolsPage should render ToolSection component",
  );
  assert.ok(
    source.includes("useToolsPageActions("),
    "ToolsPage should use useToolsPageActions hook",
  );
  assert.ok(
    !source.includes("async function handle"),
    "ToolsPage should not keep inline async action handlers",
  );
  assert.ok(
    !source.includes("installedTools.map(") && !source.includes("uninstalledTools.map("),
    "ToolsPage should not inline map tool sections",
  );
});

test("ToolCard renders path diagnostics from setup status", () => {
  const source = readToolCardSource();

  assert.ok(
    source.includes("buildToolPathDiagnostics("),
    "ToolCard should derive diagnostics via buildToolPathDiagnostics",
  );
  assert.ok(
    !source.includes("tool.skillsDirExists"),
    "ToolCard should avoid in-component path diagnostic branching",
  );
  assert.ok(
    source.includes("skillsPathLabel") && source.includes("rulesPathLabel"),
    "ToolCard should render labels from diagnostics helper",
  );
});
