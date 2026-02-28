# Remaining Risk Remediation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Close all remaining unhandled/partially handled risk items (R12, R5, R10, R8, R3, R11, R13) in a fixed sequence without repeated decision pauses.

**Architecture:** Use a risk-first sequence: fix cross-platform breakage first, then harden path strategy, then address scalability, then raise test coverage, then improve data interpretation, finally align documentation and security policy. Each task follows TDD with a clear regression guard.

**Tech Stack:** Rust (Tauri 2), React 19 + TypeScript, Node test runner (`tsx --test`), Cargo tests.

---

## Fixed Execution Order

1. `R12` Claude Hook Windows compatibility hardening
2. `R5` Built-in tool path validation and diagnostics loop
3. `R10` Log/stat scalability (bounded-memory query path)
4. `R8` Frontend coverage expansion for critical flows
5. `R3` Dashboard reliability semantics (certainty vs estimate)
6. `R11` V3 doc/implementation drift correction
7. `R13` CSP tightening (safe, staged)

---

### Task 1: R12 Windows Hook Compatibility

**Files:**
- Modify: `src-tauri/src/setup/rule_hook_ops.rs`
- Modify: `src-tauri/src/setup/tests.rs`
- (Optional) Create: `src-tauri/src/setup/hook_script_templates.rs`

**Step 1: Write failing tests**
- Add tests proving generated Claude hook on Windows path does not depend on `bash`/`jq`.
- Add tests proving injected command is Windows-native (`powershell` path/script) on Windows builds.

**Step 2: Run targeted tests (expect fail)**
- Run: `cargo test setup::tests::setup_apply_configures_claude_hook -- --nocapture`
- Run: `cargo test setup::tests::set_tool_tracking_disabled_removes_claude_hook -- --nocapture`

**Step 3: Implement minimal fix**
- Generate platform-specific hook command/script.
- On Windows, parse stdin JSON using PowerShell `ConvertFrom-Json` and append log without jq.
- Keep non-Windows behavior valid, remove hard jq dependency where feasible.

**Step 4: Verify pass**
- Run all `setup` tests and full `cargo test -- --nocapture`.

**Step 5: Commit**
- `git add src-tauri/src/setup/rule_hook_ops.rs src-tauri/src/setup/tests.rs`
- `git commit -m "fix(setup): make claude hook windows-compatible without jq dependency"`

---

### Task 2: R5 Path Validation and Diagnostics

**Files:**
- Modify: `src-tauri/src/setup/tool_catalog.rs`
- Modify: `src-tauri/src/setup/status_aggregation.rs`
- Modify: `src-tauri/src/setup/types.rs`
- Modify: `src-tauri/src/setup/tests.rs`
- Modify: `src/pages/tools/ToolCard.tsx`
- Modify: `src/pages/ToolsPage.tsx`

**Step 1: Write failing tests**
- Add backend tests for explicit path diagnostic fields (exists/readable/writable/rules-path-state).
- Add frontend structure assertions for path diagnostic rendering.

**Step 2: Run tests (expect fail)**
- `cargo test setup::tests::setup_status_ -- --nocapture` (target new tests)
- `npx tsx --test test/toolsPageStructure.test.ts`

**Step 3: Implement**
- Extend `ToolStatus` with diagnostics needed to close validation loop.
- Calculate diagnostics during `setup_status_with_home`.
- Surface diagnostic badges/messages in tools UI.

**Step 4: Verify**
- `cargo test -- --nocapture`
- `npx tsx --test test/*.test.ts`
- `npm run build`

**Step 5: Commit**
- `git add src-tauri/src/setup src/pages/ToolsPage.tsx src/pages/tools/ToolCard.tsx test/toolsPageStructure.test.ts`
- `git commit -m "feat(setup): add tool path diagnostics and expose in tools page"`

---

### Task 3: R10 Log/Stats Scalability

**Files:**
- Modify: `src-tauri/src/logs.rs`
- Modify: `src-tauri/src/stats.rs`
- Modify: `src-tauri/src/logs.rs` tests
- Modify: `src-tauri/src/stats.rs` tests

**Step 1: Write failing tests**
- Add tests for large log file behavior (memory-safe scan path, pagination correctness, stable ordering).
- Add tests ensuring stats logic can operate via streaming aggregation path.

**Step 2: Run tests (expect fail)**
- `cargo test logs::tests:: -- --nocapture`
- `cargo test stats::tests:: -- --nocapture`

**Step 3: Implement**
- Refactor log reading into streaming filter/pagination helper.
- Avoid unnecessary full in-memory copies where possible.
- Ensure deterministic ordering and bounded limits.

**Step 4: Verify**
- `cargo test -- --nocapture`

**Step 5: Commit**
- `git add src-tauri/src/logs.rs src-tauri/src/stats.rs`
- `git commit -m "perf(logs): introduce streaming-friendly log and stats processing"`

---

### Task 4: R8 Frontend Coverage Expansion

**Files:**
- Create: `test/toolsActions.test.ts`
- Create: `test/logsPage.test.ts`
- Create: `test/dashboardSemantics.test.ts`
- Modify: `test/toolsPageStructure.test.ts`

**Step 1: Write failing tests first**
- Cover tools actions: toggle auto/tracking, path save validation, custom tool form validation.
- Cover logs pagination/filtering state behavior.
- Cover dashboard reliability text/badge behavior.

**Step 2: Run tests (expect fail)**
- `npx tsx --test test/toolsActions.test.ts`
- `npx tsx --test test/logsPage.test.ts`
- `npx tsx --test test/dashboardSemantics.test.ts`

**Step 3: Implement minimal production updates**
- Only add code needed for test pass; do not redesign unrelated UI.

**Step 4: Verify**
- `npx tsx --test test/*.test.ts`
- `npm run build`

**Step 5: Commit**
- `git add test src/pages`
- `git commit -m "test(frontend): expand coverage for tools logs and dashboard flows"`

---

### Task 5: R3 Dashboard Reliability Semantics

**Files:**
- Modify: `src-tauri/src/stats.rs`
- Modify: `src/api/tauri.ts`
- Modify: `src/pages/DashboardPage.tsx`
- Modify: `src/i18n/*` message files
- Modify: related tests

**Step 1: Write failing tests**
- Add test asserting stats payload includes reliability semantics (e.g. best-effort flag/notes).
- Add UI test for rendering reliability hint.

**Step 2: Run tests (expect fail)**
- Run targeted stats and dashboard tests.

**Step 3: Implement**
- Add explicit reliability metadata in API contract.
- Render reliability note/badge near KPI/unused metrics.

**Step 4: Verify**
- `cargo test -- --nocapture`
- `npx tsx --test test/*.test.ts`
- `npm run build`

**Step 5: Commit**
- `git add src-tauri/src/stats.rs src/api/tauri.ts src/pages/DashboardPage.tsx src/i18n test`
- `git commit -m "feat(dashboard): expose and render reliability semantics for stats"`

---

### Task 6: R11 V3 Doc/Implementation Drift

**Files:**
- Modify: `../MySkills Manager 独立桌面客户端：产品与技术方案 (V3).md`
- (Optional) Create: `docs/adr/` note for intentional deviations

**Step 1: Write validation checklist**
- Enumerate each drifted claim (UI stack, logs virtualization, command contracts).

**Step 2: Apply doc corrections**
- Mark implemented reality vs planned future state.
- Keep dates and current architecture explicit.

**Step 3: Verify**
- Manual grep checks for corrected sections.

**Step 4: Commit**
- `git add ../MySkills Manager 独立桌面客户端：产品与技术方案 (V3).md docs/adr`
- `git commit -m "docs(v3): reconcile implementation drift and annotate roadmap deltas"`

---

### Task 7: R13 CSP Tightening

**Files:**
- Modify: `src-tauri/tauri.conf.json`
- Modify: any frontend files requiring CSP-safe adjustments
- Add tests/manual validation checklist

**Step 1: Write failing validation step**
- Define runtime checks for script/style behavior after CSP tightening.

**Step 2: Implement staged tightening**
- First remove unsafe script allowance if safe.
- Then reduce style allowances with explicit justification for any remainder.

**Step 3: Verify**
- `npm run build`
- Launch app smoke test with critical pages
- `cargo test -- --nocapture`

**Step 4: Commit**
- `git add src-tauri/tauri.conf.json src`
- `git commit -m "security(csp): tighten tauri CSP with staged compatibility checks"`

---

## Global Verification Gate After Each Task

- `npx tsx --test test/*.test.ts`
- `npm run build`
- `cargo test -- --nocapture`

If any command fails, stop progression and fix immediately before next task.

