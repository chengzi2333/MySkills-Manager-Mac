# MySkills Manager V3.1 (Tauri)

This directory is the active V3.1 implementation track (`Tauri 2.0 + Rust + React`).

## Current Status

- F1: Skill list is available (`skills_list`)
- F2: Skill editor is available with Monaco (`skills_get_content`, `skills_save_content`)
- F3: Dashboard is available (`stats_get`)
- F4: Log viewer is available (`logs_get`)
- F5: Git integration is available (`git_status`, `git_commit`, `git_push`)
- F6: Global rules page is available (`rules_get`, `rules_save`)
- F7: In progress (`setup_status` + `setup_apply` + custom tools + tracking rule injection/hook hardening)
- F8: Not started

## Rust Commands Available

- `app_ping`
- `skills_list`
- `skills_get_content`
- `skills_save_content`
- `stats_get`
- `logs_get`
- `rules_get`
- `rules_save`
- `git_status`
- `git_commit`
- `git_push`
- `setup_status`
- `setup_apply`
- `setup_get_custom_tools`
- `setup_add_custom_tool`
- `setup_remove_custom_tool`

## Quick Start

```bash
npm install

# frontend checks
npm run lint
npm run build

# rust tests
C:\Users\Keith\.cargo\bin\cargo.exe test --manifest-path src-tauri/Cargo.toml

# dev app
C:\Users\Keith\.cargo\bin\cargo.exe tauri dev

# debug bundle
C:\Users\Keith\.cargo\bin\cargo.exe tauri build --debug
```

## Notes

- Default skills root: `~/my-skills`
- Override root path with `MYSKILLS_ROOT_DIR`
- Dashboard currently bundles ECharts in main chunk; split optimization is queued
