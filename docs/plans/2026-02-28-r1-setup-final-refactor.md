# R1 Setup Module Final Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete remaining R1 work by removing the last large orchestration and test bulk from `setup.rs` while preserving behavior.

**Architecture:** Keep `setup.rs` as a facade for public API/types/commands, and move heavy orchestration to focused submodules under `src-tauri/src/setup/`. Extract apply/sync orchestration and move tests to dedicated `setup/tests.rs` so production code and tests are cleanly separated.

**Tech Stack:** Rust, Tauri command layer, existing unit tests, `cargo test`.

---

### Task 1: Extract Apply Orchestration Module

**Files:**
- Create: `src-tauri/src/setup/apply_engine.rs`
- Modify: `src-tauri/src/setup.rs`
- Test: `src-tauri/src/setup.rs` (structure guard test)

**Step 1: Write the failing test**

Add a structure guard test in `setup.rs` asserting these signatures are not present in `setup.rs` main section:
- `fn is_skill_enabled_for_tool(...)`
- `fn is_tracking_enabled_for_tool(...)`
- `pub fn sync_saved_skill_to_copy_tools_with_home(...)`
- `pub fn apply_setup_with_paths(...)`

**Step 2: Run test to verify it fails**

Run: `cargo test apply_engine_must_be_extracted_from_setup_module -- --nocapture`
Expected: FAIL because those functions still exist in `setup.rs`.

**Step 3: Write minimal implementation**

- Create `setup/apply_engine.rs` and move orchestration logic there.
- Keep public API stable by delegating from `setup.rs` wrappers.
- Wire imports from existing modules (`sync_ops`, `status_probe`, `rule_hook_ops`, `config_store`, `tool_catalog`).

**Step 4: Run test to verify it passes**

Run: `cargo test apply_engine_must_be_extracted_from_setup_module -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint (optional in this session)**

`git add src-tauri/src/setup.rs src-tauri/src/setup/apply_engine.rs`

---

### Task 2: Move Setup Tests Into Dedicated Test Module

**Files:**
- Create: `src-tauri/src/setup/tests.rs`
- Modify: `src-tauri/src/setup.rs`
- Test: `src-tauri/src/setup/tests.rs`

**Step 1: Write the failing test**

Add a structure guard test in `setup.rs` asserting `setup.rs` does not contain inline `mod tests {` body.

**Step 2: Run test to verify it fails**

Run: `cargo test setup_tests_must_be_moved_to_dedicated_module -- --nocapture`
Expected: FAIL while inline tests still exist.

**Step 3: Write minimal implementation**

- Move the existing `#[cfg(test)] mod tests { ... }` body into `setup/tests.rs`.
- Replace inline test block in `setup.rs` with:
  - `#[cfg(test)]`
  - `mod tests;`

**Step 4: Run test to verify it passes**

Run: `cargo test setup_tests_must_be_moved_to_dedicated_module -- --nocapture`
Expected: PASS.

**Step 5: Commit checkpoint (optional in this session)**

`git add src-tauri/src/setup.rs src-tauri/src/setup/tests.rs`

---

### Task 3: Full Verification Gate

**Files:**
- Verify only

**Step 1: Backend full tests**

Run: `cargo test -- --nocapture` in `src-tauri`
Expected: all pass.

**Step 2: Frontend unit tests**

Run: `npx tsx --test test/*.test.ts`
Expected: all pass.

**Step 3: Frontend build**

Run: `npm run build`
Expected: success (existing bundle-size warning acceptable).

**Step 4: Sanity snapshot**

Run:
- `git status --short`
- `rg -n "mod apply_engine|mod tests;|apply_engine_must_be_extracted_from_setup_module|setup_tests_must_be_moved_to_dedicated_module" src-tauri/src/setup.rs`

Expected: New module hooks and structure tests present.
