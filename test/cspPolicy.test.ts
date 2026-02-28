import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

function readCsp(): string {
  const configPath = path.resolve(process.cwd(), "src-tauri/tauri.conf.json");
  const raw = fs.readFileSync(configPath, "utf8");
  const parsed = JSON.parse(raw) as {
    app?: { security?: { csp?: string } };
  };
  return parsed.app?.security?.csp ?? "";
}

function getDirectiveValue(csp: string, directive: string): string {
  const match = csp.match(new RegExp(`${directive}\\s+([^;]+)`));
  return match?.[1] ?? "";
}

test("tauri CSP disallows unsafe-inline scripts", () => {
  const csp = readCsp();
  const scriptSrc = getDirectiveValue(csp, "script-src");

  assert.ok(scriptSrc.length > 0, "script-src must be present in CSP");
  assert.ok(scriptSrc.includes("'self'"), "script-src should keep self origin");
  assert.ok(
    !scriptSrc.includes("'unsafe-inline'"),
    "script-src should not include unsafe-inline",
  );
});

test("tauri CSP disallows unsafe-inline styles", () => {
  const csp = readCsp();
  const styleSrc = getDirectiveValue(csp, "style-src");

  assert.ok(styleSrc.length > 0, "style-src must be present in CSP");
  assert.ok(styleSrc.includes("'self'"), "style-src should keep self origin");
  assert.ok(
    !styleSrc.includes("'unsafe-inline'"),
    "style-src should not include unsafe-inline",
  );
});
