# Built-in Tool Path Validation Matrix

Date: 2026-02-28

## Purpose

`setup_path_validation_matrix` provides a runtime validation matrix for built-in tool path assumptions.  
It supplements static defaults with machine-local evidence (exists/writable/selected candidate).

## Output Model

- `name`, `id`: built-in tool identity.
- `selectedSkillsDir`, `selectedRulesPath`, `pathSource`: currently selected path resolution.
- `selectedCandidateExists`: whether selected skills path exists on disk.
- `needsManualReview`: `true` when selected path is missing and should be manually verified.
- `candidates[]`: all known candidate paths for the tool with per-path diagnostics:
  - `skillsDirExists`, `skillsDirWritable`
  - `rulesPathExists`, `rulesPathWritable`
  - `selected`

## Operational Rule

Treat `needsManualReview = true` as a blocking signal before claiming that built-in path mapping is validated for that tool.

## Engineering Notes

- Command: `setup_path_validation_matrix`
- Rust entrypoint: `src-tauri/src/setup.rs`
- Logic module: `src-tauri/src/setup/path_validation.rs`
