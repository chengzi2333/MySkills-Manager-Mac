import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

function readSkillsPageSource() {
  const pagePath = path.resolve(process.cwd(), "src/pages/SkillsPage.tsx");
  return fs.readFileSync(pagePath, "utf8");
}

test("SkillsPage provides conflict detail, filtering, and resolve actions", () => {
  const source = readSkillsPageSource();

  assert.ok(
    source.includes("setupGetSkillConflictDetail("),
    "SkillsPage should request conflict detail when user opens a conflict",
  );
  assert.ok(
    source.includes("setupResolveSkillConflict("),
    "SkillsPage should call resolve API when user chooses a source",
  );
  assert.ok(
    source.includes("handleOpenConflictResolver"),
    "SkillsPage should expose a dedicated conflict detail action",
  );
  assert.ok(
    source.includes("handleResolveConflict"),
    "SkillsPage should expose a dedicated conflict resolve action",
  );
  assert.ok(
    source.includes("设为基准"),
    "SkillsPage should offer a primary action to set selected source as baseline",
  );
  assert.ok(
    source.includes("!variant.hashMatchesMySkills"),
    "SkillsPage conflict drawer should hide variants that already match baseline",
  );
  assert.ok(
    source.includes("buildSkillDiff("),
    "SkillsPage should build readable diff between baseline and conflicting variant",
  );
  assert.ok(
    source.includes("conflictViewMode"),
    "SkillsPage should keep explicit conflict diff view mode state",
  );
  assert.ok(
    source.includes("仅看变更"),
    "SkillsPage should offer changed-lines mode in conflict drawer",
  );
  assert.ok(
    source.includes("完整内容"),
    "SkillsPage should offer full-content mode in conflict drawer",
  );
});
