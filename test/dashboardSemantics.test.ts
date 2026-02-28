import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

function readDashboardSource() {
  const pagePath = path.resolve(process.cwd(), "src/pages/DashboardPage.tsx");
  return fs.readFileSync(pagePath, "utf8");
}

test("DashboardPage surfaces stats reliability note", () => {
  const source = readDashboardSource();

  assert.ok(
    source.includes("reliability_note"),
    "Dashboard should render reliability_note from stats payload",
  );
});

test("DashboardPage avoids inline style attributes", () => {
  const source = readDashboardSource();

  assert.ok(
    !source.includes("style={{"),
    "Dashboard should avoid inline style attributes for CSP hardening",
  );
});
