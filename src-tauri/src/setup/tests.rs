use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::*;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_home() -> PathBuf {
    let mut root = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    root.push(format!("myskills-tauri-setup-test-{ts}-{n}"));
    root
}

fn find_tool<'a>(list: &'a [ToolStatus], id: &str) -> &'a ToolStatus {
    list.iter()
        .find(|item| item.id == id)
        .expect("find tool status")
}

#[test]
fn custom_tool_registry_adds_and_removes_tool() {
    let home = temp_home();
    let custom = CustomTool {
        name: "Aider".to_string(),
        id: "aider".to_string(),
        skills_dir: home
            .join(".aider")
            .join("skills")
            .to_string_lossy()
            .to_string(),
        rules_file: Some(
            home.join(".aider")
                .join("AGENTS.md")
                .to_string_lossy()
                .to_string(),
        ),
        icon: None,
    };

    let add_result = add_custom_tool_with_home(&home, custom).expect("add custom tool");
    assert!(add_result.success);

    let list = get_custom_tools_with_home(&home).expect("get custom tools");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "aider");

    let remove_result = remove_custom_tool_with_home(&home, "aider").expect("remove custom");
    assert!(remove_result.success);

    let after_remove = get_custom_tools_with_home(&home).expect("get custom tools");
    assert!(after_remove.is_empty());
}

#[test]
fn setup_status_reads_codex_configuration_and_copy_sync_mode() {
    let home = temp_home();
    let codex_skills = home.join(".codex").join("skills");
    fs::create_dir_all(&codex_skills).expect("create codex skills");
    fs::create_dir_all(codex_skills.join("skill-a")).expect("create skill-a dir");
    fs::write(
        codex_skills.join("skill-a").join("SKILL.md"),
        "---\nname: skill-a\n---\n",
    )
    .expect("write skill-a");
    fs::create_dir_all(codex_skills.join("skill-b")).expect("create skill-b dir");
    fs::write(
        codex_skills.join("skill-b").join("SKILL.md"),
        "---\nname: skill-b\n---\n",
    )
    .expect("write skill-b");
    fs::create_dir_all(codex_skills.join(".system")).expect("create non-skill dir");

    let codex_rules = home.join(".codex").join("AGENTS.md");
    fs::write(
        &codex_rules,
        "<!-- [MySkills Manager] Skill 使用追踪规则 -->\n",
    )
    .expect("write rules file");

    let list = setup_status_with_home(&home).expect("setup status");
    let codex = find_tool(&list, "codex");
    assert!(codex.exists);
    assert!(codex.configured);
    assert_eq!(codex.synced_skills, 2);
    assert_eq!(codex.sync_mode, "copy");
    assert!(!codex.hook_configured);
    assert!(!codex.is_custom);
}

#[test]
fn setup_status_exposes_last_sync_time_for_copy_mode() {
    let home = temp_home();
    let codex_skills = home.join(".codex").join("skills");
    fs::create_dir_all(codex_skills.join("code-review")).expect("create codex skill dir");
    fs::write(
        codex_skills.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write codex skill");

    let list = setup_status_with_home(&home).expect("setup status");
    let codex = find_tool(&list, "codex");
    assert_eq!(codex.sync_mode, "copy");
    assert!(codex.last_sync_time.is_some());
}

#[test]
fn setup_status_autodetects_opencode_legacy_path() {
    let home = temp_home();
    let legacy_skills = home.join(".opencode").join("skills");
    fs::create_dir_all(legacy_skills.join("workflow")).expect("create legacy skill dir");
    fs::write(
        legacy_skills.join("workflow").join("SKILL.md"),
        "---\nname: workflow\n---\n",
    )
    .expect("write legacy skill");

    let list = setup_status_with_home(&home).expect("setup status");
    let opencode = find_tool(&list, "opencode");

    assert!(opencode.exists);
    assert_eq!(opencode.skills_dir, legacy_skills.to_string_lossy());
    assert_eq!(opencode.path_source, "auto-detected");
}

#[test]
fn setup_status_marks_builtin_override_path_source() {
    let home = temp_home();
    let custom_codex_skills = home.join("custom").join("codex-skills");
    let custom_codex_skills_str = custom_codex_skills.to_string_lossy().to_string();
    update_tool_paths_with_home(&home, "codex", &custom_codex_skills_str, None)
        .expect("set codex override");

    let list = setup_status_with_home(&home).expect("setup status");
    let codex = find_tool(&list, "codex");

    assert_eq!(codex.skills_dir, custom_codex_skills.to_string_lossy());
    assert_eq!(codex.path_source, "override");
}

#[test]
fn setup_status_detects_claude_hook() {
    let home = temp_home();
    fs::create_dir_all(home.join(".claude")).expect("create claude dir");
    fs::write(
  home.join(".claude").join("settings.json"),
  r#"{"hooks":{"PostToolUse":[{"hooks":[{"command":"bash ~/.claude/hooks/skill-tracker.sh"}]}]}}"#,
)
.expect("write settings");

    let list = setup_status_with_home(&home).expect("setup status");
    let claude = find_tool(&list, "claude-code");
    assert!(claude.hook_configured);
}

#[test]
fn setup_apply_syncs_skills_to_codex() {
    let home = temp_home();
    let skills_root = temp_home();

    fs::create_dir_all(home.join(".codex")).expect("create codex parent");
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");
    fs::create_dir_all(skills_root.join("debug-helper")).expect("create skill dir");
    fs::write(
        skills_root.join("debug-helper").join("SKILL.md"),
        "---\nname: debug-helper\n---\n",
    )
    .expect("write skill");

    let result = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");
    assert_eq!(result.len(), 1);
    assert!(result[0].success);
    assert_eq!(result[0].synced_count, 2);
    assert!(result[0].sync_mode == "symlink" || result[0].sync_mode == "copy");

    assert!(home
        .join(".codex")
        .join("skills")
        .join("code-review")
        .join("SKILL.md")
        .exists());
    assert!(home
        .join(".codex")
        .join("skills")
        .join("debug-helper")
        .join("SKILL.md")
        .exists());
}

#[test]
fn setup_apply_respects_per_tool_skill_config() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(home.join(".codex")).expect("create codex parent");
    fs::create_dir_all(home.join(".myskills-manager")).expect("create app config dir");

    fs::create_dir_all(skills_root.join("code-review")).expect("create code-review skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write code-review skill");

    fs::create_dir_all(skills_root.join("debug-helper"))
        .expect("create debug-helper skill dir");
    fs::write(
        skills_root.join("debug-helper").join("SKILL.md"),
        "---\nname: debug-helper\n---\n",
    )
    .expect("write debug-helper skill");

    let sync_config = serde_json::json!({
      "syncMode": "symlink",
      "skills": [
        { "skillName": "code-review", "enabledTools": ["codex"] }
      ]
    });
    fs::write(
        home.join(".myskills-manager").join("sync-config.json"),
        serde_json::to_string(&sync_config).expect("serialize sync config"),
    )
    .expect("write sync config");

    let result = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");

    assert!(result[0].success);
    assert_eq!(result[0].synced_count, 1);
    assert!(home
        .join(".codex")
        .join("skills")
        .join("code-review")
        .join("SKILL.md")
        .exists());
    assert!(!home
        .join(".codex")
        .join("skills")
        .join("debug-helper")
        .join("SKILL.md")
        .exists());
}

#[test]
fn setup_apply_returns_error_for_unknown_tool() {
    let home = temp_home();
    let skills_root = temp_home();

    let result = apply_setup_with_paths(&home, &skills_root, &["unknown".to_string()], None)
        .expect("apply setup");
    assert_eq!(result.len(), 1);
    assert!(!result[0].success);
    assert_eq!(result[0].tool, "unknown");
    assert!(result[0].error.is_some());
}

#[test]
fn setup_status_includes_custom_tools_from_config() {
    let home = temp_home();
    let custom_skills_dir = home.join(".aider").join("skills");
    let custom_rules = home.join(".aider").join("AGENTS.md");
    fs::create_dir_all(custom_skills_dir.parent().expect("custom parent"))
        .expect("create custom parent");
    fs::create_dir_all(&custom_skills_dir).expect("create custom skills");
    fs::create_dir_all(home.join(".myskills-manager")).expect("create app config dir");
    let config = serde_json::json!([{
      "name": "Aider",
      "id": "aider",
      "skillsDir": custom_skills_dir.to_string_lossy().to_string(),
      "rulesFile": custom_rules.to_string_lossy().to_string()
    }]);
    fs::write(
        home.join(".myskills-manager").join("custom-tools.json"),
        serde_json::to_string(&config).expect("serialize config"),
    )
    .expect("write custom tools");

    let list = setup_status_with_home(&home).expect("setup status");
    let aider = find_tool(&list, "aider");
    assert!(aider.is_custom);
    assert_eq!(aider.name, "Aider");
    assert_eq!(aider.skills_dir, custom_skills_dir.to_string_lossy());
    assert_eq!(aider.rules_path, custom_rules.to_string_lossy());
}

#[test]
fn local_skills_overview_detects_duplicates_across_tools() {
    let home = temp_home();

    fs::create_dir_all(home.join(".codex").join("skills").join("shared-skill"))
        .expect("create codex shared skill");
    fs::write(
        home.join(".codex")
            .join("skills")
            .join("shared-skill")
            .join("SKILL.md"),
        "---\nname: shared-skill\n---\n",
    )
    .expect("write codex shared skill");

    fs::create_dir_all(home.join(".codex").join("skills").join("codex-only"))
        .expect("create codex only skill");
    fs::write(
        home.join(".codex")
            .join("skills")
            .join("codex-only")
            .join("SKILL.md"),
        "---\nname: codex-only\n---\n",
    )
    .expect("write codex only skill");

    fs::create_dir_all(home.join(".claude").join("skills").join("shared-skill"))
        .expect("create claude shared skill");
    fs::write(
        home.join(".claude")
            .join("skills")
            .join("shared-skill")
            .join("SKILL.md"),
        "---\nname: shared-skill\n---\n",
    )
    .expect("write claude shared skill");

    let overview = local_skills_overview_with_home(&home).expect("local skills overview");
    assert_eq!(overview.tools.len(), 2);
    assert_eq!(overview.total_skills, 3);
    assert_eq!(overview.unique_skills, 2);
    assert_eq!(overview.duplicate_names, vec!["shared-skill".to_string()]);
    assert_eq!(overview.matched_in_my_skills, 0);
    assert_eq!(overview.missing_in_my_skills, 2);
    assert_eq!(overview.conflict_with_my_skills, 0);

    let codex = overview
        .tools
        .iter()
        .find(|tool| tool.tool_id == "codex")
        .expect("find codex overview");
    let shared = codex
        .skills
        .iter()
        .find(|skill| skill.name == "shared-skill")
        .expect("find shared skill");
    assert!(shared.duplicate_across_tools);
    assert!(!shared.in_my_skills);
    assert!(!shared.hash_matches_my_skills);
    assert!(!shared.hash_conflicts_my_skills);
}

#[test]
fn local_skills_overview_marks_match_conflict_and_missing_vs_my_skills() {
    let home = temp_home();

    fs::create_dir_all(home.join("my-skills").join("shared-skill")).expect("create my shared");
    fs::write(
        home.join("my-skills").join("shared-skill").join("SKILL.md"),
        "---\nname: shared-skill\nversion: 1\n---\n",
    )
    .expect("write my shared");

    fs::create_dir_all(home.join("my-skills").join("aligned-skill"))
        .expect("create my aligned");
    fs::write(
        home.join("my-skills")
            .join("aligned-skill")
            .join("SKILL.md"),
        "---\nname: aligned-skill\nversion: 1\n---\n",
    )
    .expect("write my aligned");

    fs::create_dir_all(home.join(".codex").join("skills").join("shared-skill"))
        .expect("create codex shared");
    fs::write(
        home.join(".codex")
            .join("skills")
            .join("shared-skill")
            .join("SKILL.md"),
        "---\nname: shared-skill\nversion: 1\n---\n",
    )
    .expect("write codex shared");

    fs::create_dir_all(home.join(".codex").join("skills").join("aligned-skill"))
        .expect("create codex aligned");
    fs::write(
        home.join(".codex")
            .join("skills")
            .join("aligned-skill")
            .join("SKILL.md"),
        "---\nname: aligned-skill\nversion: 1\n---\n",
    )
    .expect("write codex aligned");

    fs::create_dir_all(home.join(".codex").join("skills").join("new-skill"))
        .expect("create codex new");
    fs::write(
        home.join(".codex")
            .join("skills")
            .join("new-skill")
            .join("SKILL.md"),
        "---\nname: new-skill\n---\n",
    )
    .expect("write codex new");

    fs::create_dir_all(home.join(".claude").join("skills").join("shared-skill"))
        .expect("create claude shared");
    fs::write(
        home.join(".claude")
            .join("skills")
            .join("shared-skill")
            .join("SKILL.md"),
        "---\nname: shared-skill\nversion: 2\n---\n",
    )
    .expect("write claude shared");

    let overview = local_skills_overview_with_home(&home).expect("local skills overview");
    assert_eq!(overview.duplicate_names, vec!["shared-skill".to_string()]);
    assert_eq!(overview.unique_skills, 3);
    assert_eq!(overview.matched_in_my_skills, 1);
    assert_eq!(overview.missing_in_my_skills, 1);
    assert_eq!(overview.conflict_with_my_skills, 1);

    let codex = overview
        .tools
        .iter()
        .find(|tool| tool.tool_id == "codex")
        .expect("find codex");
    let codex_shared = codex
        .skills
        .iter()
        .find(|skill| skill.name == "shared-skill")
        .expect("find codex shared");
    assert!(codex_shared.in_my_skills);
    assert!(codex_shared.hash_matches_my_skills);
    assert!(!codex_shared.hash_conflicts_my_skills);

    let codex_missing = codex
        .skills
        .iter()
        .find(|skill| skill.name == "new-skill")
        .expect("find codex new");
    assert!(!codex_missing.in_my_skills);
    assert!(!codex_missing.hash_matches_my_skills);
    assert!(!codex_missing.hash_conflicts_my_skills);

    let claude = overview
        .tools
        .iter()
        .find(|tool| tool.tool_id == "claude-code")
        .expect("find claude");
    let claude_shared = claude
        .skills
        .iter()
        .find(|skill| skill.name == "shared-skill")
        .expect("find claude shared");
    assert!(claude_shared.in_my_skills);
    assert!(!claude_shared.hash_matches_my_skills);
    assert!(claude_shared.hash_conflicts_my_skills);
}

#[test]
fn setup_apply_syncs_skills_to_custom_tool() {
    let home = temp_home();
    let skills_root = temp_home();
    let custom_skills_dir = home.join(".aider").join("skills");
    fs::create_dir_all(custom_skills_dir.parent().expect("custom parent"))
        .expect("create custom parent");
    fs::create_dir_all(skills_root.join("debug-helper")).expect("create skill dir");
    fs::write(
        skills_root.join("debug-helper").join("SKILL.md"),
        "---\nname: debug-helper\n---\n",
    )
    .expect("write skill");
    fs::create_dir_all(home.join(".myskills-manager")).expect("create app config dir");
    let config = serde_json::json!([{
      "name": "Aider",
      "id": "aider",
      "skillsDir": custom_skills_dir.to_string_lossy().to_string()
    }]);

    fs::write(
        home.join(".myskills-manager").join("custom-tools.json"),
        serde_json::to_string(&config).expect("serialize config"),
    )
    .expect("write custom tools");

    let result = apply_setup_with_paths(&home, &skills_root, &["aider".to_string()], None)
        .expect("apply setup");
    assert_eq!(result.len(), 1);
    assert!(result[0].success);
    assert_eq!(result[0].tool, "aider");
    assert_eq!(result[0].synced_count, 1);
    assert!(custom_skills_dir
        .join("debug-helper")
        .join("SKILL.md")
        .exists());
}

#[test]
fn setup_apply_injects_rules_once_for_codex() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let first = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");
    assert!(first[0].success);

    let second = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");
    assert!(second[0].success);

    let rules_content =
        fs::read_to_string(home.join(".codex").join("AGENTS.md")).expect("read codex rules");
    let marker_count = rules_content.matches("[MySkills Manager]").count();
    assert_eq!(marker_count, 1);
    assert!(rules_content.contains("codex"));
}

#[test]
fn setup_apply_configures_claude_hook() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let result =
        apply_setup_with_paths(&home, &skills_root, &["claude-code".to_string()], None)
            .expect("apply setup");
    assert!(result[0].success);

    let hook_script = home.join(".claude").join("hooks").join("skill-tracker.sh");
    assert!(hook_script.exists());

    let settings = fs::read_to_string(home.join(".claude").join("settings.json"))
        .expect("read claude settings");
    assert!(settings.contains("skill-tracker.sh"));
}

#[test]
fn update_tool_paths_persists_override_for_builtin_tool() {
    let home = temp_home();
    let new_skills = home.join("custom").join("codex-skills");
    let new_rules = home.join("custom").join("CODERULES.md");
    let new_skills_str = new_skills.to_string_lossy().to_string();
    let new_rules_str = new_rules.to_string_lossy().to_string();

    let result =
        update_tool_paths_with_home(&home, "codex", &new_skills_str, Some(&new_rules_str))
            .expect("update built-in tool paths");
    assert!(result.success);

    let status = setup_status_with_home(&home).expect("setup status");
    let codex = find_tool(&status, "codex");
    assert_eq!(codex.skills_dir, new_skills.to_string_lossy());
    assert_eq!(codex.rules_path, new_rules.to_string_lossy());
}

#[test]
fn update_tool_paths_updates_custom_tool_registry() {
    let home = temp_home();
    let custom = CustomTool {
        name: "Aider".to_string(),
        id: "aider".to_string(),
        skills_dir: home
            .join(".aider")
            .join("skills")
            .to_string_lossy()
            .to_string(),
        rules_file: Some(
            home.join(".aider")
                .join("AGENTS.md")
                .to_string_lossy()
                .to_string(),
        ),
        icon: None,
    };
    add_custom_tool_with_home(&home, custom).expect("add custom");

    let new_skills = home.join(".aider").join("instructions");
    let new_rules = home.join(".aider").join("AIDER.md");
    let new_skills_str = new_skills.to_string_lossy().to_string();
    let new_rules_str = new_rules.to_string_lossy().to_string();
    let result =
        update_tool_paths_with_home(&home, "aider", &new_skills_str, Some(&new_rules_str))
            .expect("update custom paths");
    assert!(result.success);

    let custom_tools = get_custom_tools_with_home(&home).expect("read custom tools");
    assert_eq!(custom_tools.len(), 1);
    assert_eq!(custom_tools[0].skills_dir, new_skills.to_string_lossy());
    assert_eq!(
        custom_tools[0].rules_file.as_deref(),
        Some(new_rules_str.as_str())
    );

    let status = setup_status_with_home(&home).expect("setup status");
    let aider = find_tool(&status, "aider");
    assert_eq!(aider.skills_dir, new_skills.to_string_lossy());
    assert_eq!(aider.rules_path, new_rules.to_string_lossy());
}

#[test]
fn setup_auto_sync_toggle_persists_in_status() {
    let home = temp_home();

    let before = setup_status_with_home(&home).expect("setup status before");
    let codex_before = find_tool(&before, "codex");
    assert!(!codex_before.auto_sync);

    let enable = set_tool_auto_sync_with_home(&home, "codex", true).expect("enable auto sync");
    assert!(enable.success);

    let after_enable = setup_status_with_home(&home).expect("setup status after enable");
    let codex_enabled = find_tool(&after_enable, "codex");
    assert!(codex_enabled.auto_sync);

    let disable =
        set_tool_auto_sync_with_home(&home, "codex", false).expect("disable auto sync");
    assert!(disable.success);

    let after_disable = setup_status_with_home(&home).expect("setup status after disable");
    let codex_disabled = find_tool(&after_disable, "codex");
    assert!(!codex_disabled.auto_sync);
}

#[test]
fn set_tool_auto_sync_rejects_unknown_tool() {
    let home = temp_home();
    let err = set_tool_auto_sync_with_home(&home, "not-a-tool", true)
        .expect_err("unknown tool should fail");
    assert!(err.contains("Tool id not supported"));
}

#[test]
fn setup_tracking_toggle_persists_in_status() {
    let home = temp_home();

    let before = setup_status_with_home(&home).expect("setup status before");
    let codex_before = find_tool(&before, "codex");
    assert!(codex_before.tracking_enabled);

    let disable =
        set_tool_tracking_enabled_with_home(&home, "codex", false).expect("disable tracking");
    assert!(disable.success);

    let after_disable = setup_status_with_home(&home).expect("setup status after disable");
    let codex_disabled = find_tool(&after_disable, "codex");
    assert!(!codex_disabled.tracking_enabled);

    let enable =
        set_tool_tracking_enabled_with_home(&home, "codex", true).expect("enable tracking");
    assert!(enable.success);

    let after_enable = setup_status_with_home(&home).expect("setup status after enable");
    let codex_enabled = find_tool(&after_enable, "codex");
    assert!(codex_enabled.tracking_enabled);
}

#[test]
fn setup_apply_removes_rules_block_when_tracking_disabled() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let rules_path = home.join(".codex").join("AGENTS.md");
    ensure_rules_injected("codex", &rules_path).expect("inject rules first");
    let with_marker = fs::read_to_string(&rules_path).expect("read rules");
    assert!(with_marker.contains("[MySkills Manager]"));

    set_tool_tracking_enabled_with_home(&home, "codex", false).expect("disable tracking");
    let result = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");
    assert!(result[0].success);

    let after = fs::read_to_string(&rules_path).expect("read rules after");
    assert!(!after.contains("[MySkills Manager]"));
}

#[test]
fn set_tool_tracking_disabled_removes_claude_hook() {
    let home = temp_home();
    ensure_claude_hook(&home).expect("ensure hook");
    assert!(home
        .join(".claude")
        .join("hooks")
        .join("skill-tracker.sh")
        .exists());

    set_tool_tracking_enabled_with_home(&home, "claude-code", false)
        .expect("disable claude tracking");

    assert!(!home
        .join(".claude")
        .join("hooks")
        .join("skill-tracker.sh")
        .exists());
}

#[test]
fn setup_apply_creates_backup_before_modifying_rules_file() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let rules_path = home.join(".codex").join("AGENTS.md");
    fs::create_dir_all(rules_path.parent().expect("rules parent")).expect("create rules dir");
    fs::write(&rules_path, "legacy rules\n").expect("write legacy rules");

    let result = apply_setup_with_paths(&home, &skills_root, &["codex".to_string()], None)
        .expect("apply setup");
    assert!(result[0].success);

    let backup_path = home.join(".codex").join("AGENTS.md.bak");
    assert!(backup_path.exists(), "expected AGENTS.md.bak to exist");
    let backup_content = fs::read_to_string(backup_path).expect("read backup");
    assert_eq!(backup_content, "legacy rules\n");
}

#[test]
fn setup_apply_rolls_back_previous_config_writes_when_later_tool_fails() {
    let home = temp_home();
    let skills_root = temp_home();
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let codex_rules = home.join(".codex").join("AGENTS.md");
    fs::create_dir_all(codex_rules.parent().expect("codex rules parent"))
        .expect("create codex rules parent");
    fs::write(&codex_rules, "codex old rules\n").expect("write old codex rules");

    let broken_rules_dir = home.join(".broken").join("rules-as-dir");
    fs::create_dir_all(&broken_rules_dir).expect("create broken rules dir");
    fs::create_dir_all(home.join(".myskills-manager")).expect("create app config dir");
    let custom_tool = serde_json::json!([{
      "name": "BrokenTool",
      "id": "broken",
      "skillsDir": home.join(".broken").join("skills").to_string_lossy().to_string(),
      "rulesFile": broken_rules_dir.to_string_lossy().to_string()
    }]);
    fs::write(
        home.join(".myskills-manager").join("custom-tools.json"),
        serde_json::to_string(&custom_tool).expect("serialize custom tools"),
    )
    .expect("write custom tools");

    let result = apply_setup_with_paths(
        &home,
        &skills_root,
        &["codex".to_string(), "broken".to_string()],
        None,
    )
    .expect("apply setup");

    assert!(!result.iter().all(|item| item.success));
    let codex_after = fs::read_to_string(&codex_rules).expect("read codex rules after");
    assert_eq!(codex_after, "codex old rules\n");
}

#[test]
fn sync_saved_skill_to_copy_tools_updates_existing_copy_targets() {
    let home = temp_home();
    let skills_root = temp_home();

    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
        skills_root.join("code-review").join("SKILL.md"),
        "---\nname: code-review\n---\nnew content\n",
    )
    .expect("write source skill");

    let codex_skill_dir = home.join(".codex").join("skills").join("code-review");
    fs::create_dir_all(&codex_skill_dir).expect("create codex skill dir");
    fs::write(
        codex_skill_dir.join("SKILL.md"),
        "---\nname: code-review\n---\nold content\n",
    )
    .expect("write old copied skill");

    let synced = sync_saved_skill_to_copy_tools_with_home(&home, &skills_root, "code-review")
        .expect("sync saved skill");

    assert_eq!(synced, 1);
    let updated = fs::read_to_string(codex_skill_dir.join("SKILL.md")).expect("read updated");
    assert!(updated.contains("new content"));
    assert!(!updated.contains("old content"));
}

#[test]
fn path_helpers_must_not_be_duplicated_across_modules() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let onboarding_source = fs::read_to_string(manifest_dir.join("src").join("onboarding.rs"))
        .expect("read onboarding.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn default_home_dir() -> PathBuf {",
        "fn default_skills_root(home: &Path) -> PathBuf {",
        "fn app_config_dir(home: &Path) -> PathBuf {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "duplicate helper in setup.rs: {signature}"
        );
        assert!(
            !onboarding_source.contains(signature),
            "duplicate helper in onboarding.rs: {signature}"
        );
    }
}

#[test]
fn tool_catalog_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "struct ToolDescriptor {",
        "fn built_in_tools(home: &Path) -> Vec<ToolDescriptor> {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "tool catalog should live outside setup.rs: {signature}"
        );
    }
}

#[test]
fn config_store_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn read_custom_tools(home: &Path) -> Result<Vec<CustomTool>, String> {",
        "fn read_tool_path_overrides(home: &Path) -> Result<Vec<ToolPathOverride>, String> {",
        "fn read_sync_config(home: &Path) -> Result<Option<SyncConfigFile>, String> {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "config store should live outside setup.rs: {signature}"
        );
    }
}

#[test]
fn status_probe_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn file_contains_marker(path: &Path) -> bool {",
        "fn detect_sync_stats(skills_dir: &Path) -> Result<(usize, String, Option<String>), String> {",
        "fn detect_claude_hook(home: &Path) -> bool {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "status probe should live outside setup.rs: {signature}"
        );
    }
}

#[test]
fn sync_ops_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn remove_if_exists(path: &Path) -> Result<(), String> {",
        "fn backup_if_exists(path: &Path) -> Result<(), String> {",
        "fn finalize_with_rollback(",
        "fn sync_skill_file(source: &Path, target: &Path) -> Result<String, String> {",
        "fn remove_skill_target(target_dir: &Path, target_file: &Path) -> Result<(), String> {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "sync ops should live outside setup.rs: {signature}"
        );
    }
}

#[test]
fn rule_and_hook_ops_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn tracking_rule_block(tool_id: &str) -> String {",
        "fn build_rule_block(tool_id: &str, rules_path: &Path) -> String {",
        "fn upsert_marker_block(existing: &str, block: &str) -> String {",
        "fn remove_marker_block(existing: &str) -> String {",
        "fn ensure_rules_injected(tool_id: &str, rules_path: &Path) -> Result<(), String> {",
        "fn ensure_rules_removed(rules_path: &Path) -> Result<(), String> {",
        "fn ensure_claude_hook(home: &Path) -> Result<(), String> {",
        "fn ensure_claude_hook_removed(home: &Path) -> Result<(), String> {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "rule/hook ops should live outside setup.rs: {signature}"
        );
    }
}

#[test]
fn apply_engine_must_be_extracted_from_setup_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    let setup_main = setup_source
        .split("#[cfg(test)]")
        .next()
        .expect("setup main section");

    for signature in [
        "fn is_skill_enabled_for_tool(",
        "fn is_tracking_enabled_for_tool(tool_id: &str, tracking_disabled_tools: &HashSet<String>) -> bool {",
    ] {
        assert!(
            !setup_main.contains(signature),
            "apply engine should live outside setup.rs: {signature}"
        );
    }
    assert!(setup_main.contains("apply_engine::sync_saved_skill_to_copy_tools_with_home("));
    assert!(setup_main.contains("apply_engine::apply_setup_with_paths("));
}

#[test]
fn setup_tests_must_be_moved_to_dedicated_module() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let setup_source =
        fs::read_to_string(manifest_dir.join("src").join("setup.rs")).expect("read setup.rs");
    assert!(
        !setup_source.contains("mod tests {"),
        "setup.rs should use #[cfg(test)] mod tests; with tests in src/setup/tests.rs"
    );
}
