use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
  pub name: String,
  pub id: String,
  pub skills_dir: String,
  pub rules_path: String,
  pub exists: bool,
  pub configured: bool,
  pub synced_skills: usize,
  pub sync_mode: String,
  pub hook_configured: bool,
  pub is_custom: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ApplyResult {
  pub tool: String,
  pub success: bool,
  pub action: String,
  pub sync_mode: String,
  pub synced_count: usize,
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CustomTool {
  pub name: String,
  pub id: String,
  pub skills_dir: String,
  pub rules_file: Option<String>,
  pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct SetupMutationResult {
  pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillSyncConfig {
  pub skill_name: String,
  pub enabled_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct SyncConfigFile {
  #[serde(default = "default_sync_mode")]
  sync_mode: String,
  #[serde(default)]
  skills: Vec<SkillSyncConfig>,
}

#[derive(Debug, Clone)]
struct ToolDescriptor {
  name: String,
  id: String,
  skills_dir: PathBuf,
  rules_path: Option<PathBuf>,
  is_custom: bool,
}

const TRACKER_BLOCK_START: &str =
  "<!-- [MySkills Manager] Skill usage tracking rule - DO NOT REMOVE -->";
const TRACKER_BLOCK_END: &str = "<!-- [/MySkills Manager] -->";
const CLAUDE_HOOK_REL_PATH: &str = ".claude/hooks/skill-tracker.sh";

fn default_sync_mode() -> String {
  "symlink".to_string()
}

fn default_home_dir() -> PathBuf {
  if let Ok(home) = std::env::var("HOME") {
    return PathBuf::from(home);
  }
  if let Ok(home) = std::env::var("USERPROFILE") {
    return PathBuf::from(home);
  }
  PathBuf::from("./")
}

fn default_skills_root(home: &Path) -> PathBuf {
  if let Ok(path) = std::env::var("MYSKILLS_ROOT_DIR") {
    return PathBuf::from(path);
  }
  home.join("my-skills")
}

fn app_config_dir(home: &Path) -> PathBuf {
  home.join(".myskills-manager")
}

fn custom_tools_file(home: &Path) -> PathBuf {
  app_config_dir(home).join("custom-tools.json")
}

fn sync_config_file(home: &Path) -> PathBuf {
  app_config_dir(home).join("sync-config.json")
}

fn built_in_tools(home: &Path) -> Vec<ToolDescriptor> {
  vec![
    ToolDescriptor {
      name: "Antigravity".to_string(),
      id: "antigravity".to_string(),
      skills_dir: home.join(".gemini").join("instructions"),
      rules_path: Some(home.join(".gemini").join("GEMINI.md")),
      is_custom: false,
    },
    ToolDescriptor {
      name: "Codex".to_string(),
      id: "codex".to_string(),
      skills_dir: home.join(".codex").join("skills"),
      rules_path: Some(home.join(".codex").join("AGENTS.md")),
      is_custom: false,
    },
    ToolDescriptor {
      name: "Claude Code".to_string(),
      id: "claude-code".to_string(),
      skills_dir: home.join(".claude").join("skills"),
      rules_path: Some(home.join(".claude").join("CLAUDE.md")),
      is_custom: false,
    },
    ToolDescriptor {
      name: "Cursor".to_string(),
      id: "cursor".to_string(),
      skills_dir: home.join(".cursor").join("rules"),
      rules_path: Some(home.join(".cursor").join("rules").join("myskills-tracker.mdc")),
      is_custom: false,
    },
    ToolDescriptor {
      name: "Windsurf".to_string(),
      id: "windsurf".to_string(),
      skills_dir: home.join(".codeium").join("windsurf").join("skills"),
      rules_path: Some(
        home
          .join(".codeium")
          .join("windsurf")
          .join("memories")
          .join("global_rules.md"),
      ),
      is_custom: false,
    },
    ToolDescriptor {
      name: "OpenCode".to_string(),
      id: "opencode".to_string(),
      skills_dir: home.join(".config").join("opencode").join("skills"),
      rules_path: Some(home.join(".config").join("opencode").join("AGENTS.md")),
      is_custom: false,
    },
  ]
}

fn read_custom_tools(home: &Path) -> Result<Vec<CustomTool>, String> {
  let path = custom_tools_file(home);
  if !path.exists() {
    return Ok(Vec::new());
  }

  let raw = fs::read_to_string(path).map_err(|e| format!("Read custom tools failed: {e}"))?;
  if raw.trim().is_empty() {
    return Ok(Vec::new());
  }

  serde_json::from_str::<Vec<CustomTool>>(&raw)
    .map_err(|e| format!("Invalid custom tools config: {e}"))
}

fn write_custom_tools(home: &Path, tools: &[CustomTool]) -> Result<(), String> {
  fs::create_dir_all(app_config_dir(home))
    .map_err(|e| format!("Create app config dir failed: {e}"))?;
  let content = serde_json::to_string_pretty(tools)
    .map_err(|e| format!("Serialize custom tools failed: {e}"))?;
  fs::write(custom_tools_file(home), format!("{content}\n"))
    .map_err(|e| format!("Write custom tools failed: {e}"))
}

fn read_sync_config(home: &Path) -> Result<Option<SyncConfigFile>, String> {
  let path = sync_config_file(home);
  if !path.exists() {
    return Ok(None);
  }

  let raw = fs::read_to_string(path).map_err(|e| format!("Read sync config failed: {e}"))?;
  if raw.trim().is_empty() {
    return Ok(None);
  }

  let parsed = serde_json::from_str::<SyncConfigFile>(&raw)
    .map_err(|e| format!("Invalid sync config: {e}"))?;
  Ok(Some(parsed))
}

fn write_sync_config(home: &Path, skills: &[SkillSyncConfig]) -> Result<(), String> {
  fs::create_dir_all(app_config_dir(home))
    .map_err(|e| format!("Create app config dir failed: {e}"))?;

  let sync_mode = read_sync_config(home)?
    .map(|cfg| cfg.sync_mode)
    .unwrap_or_else(default_sync_mode);
  let content = serde_json::to_string_pretty(&SyncConfigFile {
    sync_mode,
    skills: skills.to_vec(),
  })
  .map_err(|e| format!("Serialize sync config failed: {e}"))?;
  fs::write(sync_config_file(home), format!("{content}\n"))
    .map_err(|e| format!("Write sync config failed: {e}"))
}

fn validate_tool_id(id: &str) -> Result<String, String> {
  let normalized = id.trim().to_lowercase();
  if normalized.is_empty() {
    return Err("Tool id is required".to_string());
  }
  let valid = normalized
    .chars()
    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_');
  if !valid {
    return Err("Tool id must use [a-z0-9_-]".to_string());
  }
  Ok(normalized)
}

fn custom_tool_to_descriptor(custom: CustomTool) -> ToolDescriptor {
  ToolDescriptor {
    name: custom.name,
    id: custom.id,
    skills_dir: PathBuf::from(custom.skills_dir),
    rules_path: custom.rules_file.map(PathBuf::from),
    is_custom: true,
  }
}

fn all_tools(home: &Path) -> Result<Vec<ToolDescriptor>, String> {
  let mut tools = built_in_tools(home);
  for custom in read_custom_tools(home)? {
    tools.push(custom_tool_to_descriptor(custom));
  }
  Ok(tools)
}

fn file_contains_marker(path: &Path) -> bool {
  if !path.exists() {
    return false;
  }
  match fs::read_to_string(path) {
    Ok(content) => content.contains("[MySkills Manager]"),
    Err(_) => false,
  }
}

fn detect_sync_mode(skills_dir: &Path) -> Result<(usize, String), String> {
  if !skills_dir.exists() {
    return Ok((0, "none".to_string()));
  }

  let mut count = 0usize;
  let mut has_symlink = false;
  let entries = fs::read_dir(skills_dir).map_err(|e| format!("Read skills dir failed: {e}"))?;
  for entry in entries {
    let entry = entry.map_err(|e| format!("Read skills entry failed: {e}"))?;
    count += 1;
    let metadata = fs::symlink_metadata(entry.path())
      .map_err(|e| format!("Read skills metadata failed: {e}"))?;
    if metadata.file_type().is_symlink() {
      has_symlink = true;
    }
  }

  let mode = if count == 0 {
    "none"
  } else if has_symlink {
    "symlink"
  } else {
    "copy"
  };
  Ok((count, mode.to_string()))
}

fn detect_claude_hook(home: &Path) -> bool {
  let settings = home.join(".claude").join("settings.json");
  if !settings.exists() {
    return false;
  }
  match fs::read_to_string(settings) {
    Ok(content) => content.contains("skill-tracker.sh"),
    Err(_) => false,
  }
}

pub fn setup_status_with_home(home: &Path) -> Result<Vec<ToolStatus>, String> {
  let mut list = Vec::<ToolStatus>::new();
  for tool in all_tools(home)? {
    let exists = tool
      .skills_dir
      .parent()
      .map(|parent| parent.exists())
      .unwrap_or(false);
    let configured = tool
      .rules_path
      .as_ref()
      .map(|path| file_contains_marker(path))
      .unwrap_or(false);
    let (synced_skills, sync_mode) = detect_sync_mode(&tool.skills_dir)?;
    let hook_configured = if tool.id == "claude-code" {
      detect_claude_hook(home)
    } else {
      false
    };

    list.push(ToolStatus {
      name: tool.name.clone(),
      id: tool.id.clone(),
      skills_dir: tool.skills_dir.to_string_lossy().to_string(),
      rules_path: tool
        .rules_path
        .as_ref()
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_default(),
      exists,
      configured,
      synced_skills,
      sync_mode,
      hook_configured,
      is_custom: tool.is_custom,
    });
  }
  Ok(list)
}

#[tauri::command]
pub fn setup_status() -> Result<Vec<ToolStatus>, String> {
  setup_status_with_home(&default_home_dir())
}

pub fn get_custom_tools_with_home(home: &Path) -> Result<Vec<CustomTool>, String> {
  read_custom_tools(home)
}

pub fn add_custom_tool_with_home(home: &Path, mut tool: CustomTool) -> Result<SetupMutationResult, String> {
  tool.name = tool.name.trim().to_string();
  if tool.name.is_empty() {
    return Err("Tool name is required".to_string());
  }

  tool.id = validate_tool_id(&tool.id)?;
  tool.skills_dir = tool.skills_dir.trim().to_string();
  if tool.skills_dir.is_empty() {
    return Err("skillsDir is required".to_string());
  }
  tool.rules_file = tool.rules_file.map(|value| value.trim().to_string());
  if tool.rules_file.as_deref() == Some("") {
    tool.rules_file = None;
  }

  let mut custom = read_custom_tools(home)?;
  if built_in_tools(home).iter().any(|item| item.id == tool.id) {
    return Err("Tool id conflicts with built-in tool".to_string());
  }
  if custom.iter().any(|item| item.id == tool.id) {
    return Err("Tool id already exists".to_string());
  }

  custom.push(tool);
  custom.sort_by(|a, b| a.id.cmp(&b.id));
  write_custom_tools(home, &custom)?;
  Ok(SetupMutationResult { success: true })
}

pub fn remove_custom_tool_with_home(home: &Path, id: &str) -> Result<SetupMutationResult, String> {
  let tool_id = validate_tool_id(id)?;
  let mut custom = read_custom_tools(home)?;
  let before = custom.len();
  custom.retain(|item| item.id != tool_id);
  if custom.len() == before {
    return Ok(SetupMutationResult { success: true });
  }
  write_custom_tools(home, &custom)?;
  Ok(SetupMutationResult { success: true })
}

#[tauri::command]
pub fn setup_get_custom_tools() -> Result<Vec<CustomTool>, String> {
  get_custom_tools_with_home(&default_home_dir())
}

#[tauri::command]
pub fn setup_add_custom_tool(
  name: String,
  id: String,
  skills_dir: String,
  rules_file: Option<String>,
  icon: Option<String>,
) -> Result<SetupMutationResult, String> {
  add_custom_tool_with_home(
    &default_home_dir(),
    CustomTool {
      name,
      id,
      skills_dir,
      rules_file,
      icon,
    },
  )
}

#[tauri::command]
pub fn setup_remove_custom_tool(id: String) -> Result<SetupMutationResult, String> {
  remove_custom_tool_with_home(&default_home_dir(), &id)
}

fn remove_if_exists(path: &Path) -> Result<(), String> {
  let metadata = match fs::symlink_metadata(path) {
    Ok(metadata) => metadata,
    Err(_) => return Ok(()),
  };

  if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
    fs::remove_dir_all(path).map_err(|e| format!("Remove dir failed: {e}"))?;
  } else {
    fs::remove_file(path).map_err(|e| format!("Remove file failed: {e}"))?;
  }
  Ok(())
}

#[cfg(target_family = "windows")]
fn create_symlink_file(source: &Path, target: &Path) -> std::io::Result<()> {
  std::os::windows::fs::symlink_file(source, target)
}

#[cfg(target_family = "unix")]
fn create_symlink_file(source: &Path, target: &Path) -> std::io::Result<()> {
  std::os::unix::fs::symlink(source, target)
}

fn sync_skill_file(source: &Path, target: &Path) -> Result<String, String> {
  remove_if_exists(target)?;
  match create_symlink_file(source, target) {
    Ok(_) => Ok("symlink".to_string()),
    Err(_) => {
      fs::copy(source, target).map_err(|e| format!("Copy skill file failed: {e}"))?;
      Ok("copy".to_string())
    }
  }
}

fn tracking_rule_block(tool_id: &str) -> String {
  format!(
    r#"{start}
## Skill Usage Tracking Rule

Whenever you load and execute any Skill, silently append one JSON line to `~/my-skills/.logs/skill-usage.jsonl`.

**macOS / Linux**
```bash
mkdir -p ~/my-skills/.logs && echo "{{\"ts\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"skill\":\"<skill-name>\",\"cwd\":\"$(pwd)\",\"tool\":\"{tool_id}\"}}" >> ~/my-skills/.logs/skill-usage.jsonl
```

**Windows (PowerShell)**
```powershell
New-Item -ItemType Directory -Force -Path "$env:USERPROFILE\my-skills\.logs" | Out-Null
$ts = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
Add-Content "$env:USERPROFILE\my-skills\.logs\skill-usage.jsonl" "{{`"ts`":`"$ts`",`"skill`":`"<skill-name>`",`"cwd`":`"$(Get-Location)`",`"tool`":`"{tool_id}`"}}"
```
{end}
"#,
    start = TRACKER_BLOCK_START,
    tool_id = tool_id,
    end = TRACKER_BLOCK_END
  )
}

fn build_rule_block(tool_id: &str, rules_path: &Path) -> String {
  let is_cursor = tool_id == "cursor"
    || rules_path
      .extension()
      .map(|ext| ext.to_string_lossy().eq_ignore_ascii_case("mdc"))
      .unwrap_or(false);
  if !is_cursor {
    return tracking_rule_block(tool_id);
  }

  format!(
    "---\ndescription: MySkills Manager skill usage tracking rule\nglobs:\nalwaysApply: true\n---\n\n{}",
    tracking_rule_block(tool_id)
  )
}

fn upsert_marker_block(existing: &str, block: &str) -> String {
  let block = block.trim_end_matches('\n');
  if let Some(start) = existing.find(TRACKER_BLOCK_START) {
    if let Some(end_offset) = existing[start..].find(TRACKER_BLOCK_END) {
      let end = start + end_offset + TRACKER_BLOCK_END.len();
      let prefix = existing[..start].trim_end_matches('\n');
      let suffix = existing[end..].trim_start_matches('\n');
      let mut out = String::new();
      if !prefix.is_empty() {
        out.push_str(prefix);
        out.push_str("\n\n");
      }
      out.push_str(block);
      if !suffix.is_empty() {
        out.push_str("\n\n");
        out.push_str(suffix);
      }
      out.push('\n');
      return out;
    }
  }

  if existing.trim().is_empty() {
    return format!("{block}\n");
  }
  format!("{}\n\n{block}\n", existing.trim_end_matches('\n'))
}

fn ensure_rules_injected(tool_id: &str, rules_path: &Path) -> Result<(), String> {
  if let Some(parent) = rules_path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("Create rules dir failed: {e}"))?;
  }

  let existing = if rules_path.exists() {
    fs::read_to_string(rules_path).map_err(|e| format!("Read rules file failed: {e}"))?
  } else {
    String::new()
  };

  let next = upsert_marker_block(&existing, &build_rule_block(tool_id, rules_path));
  fs::write(rules_path, next).map_err(|e| format!("Write rules file failed: {e}"))
}

fn ensure_claude_hook(home: &Path) -> Result<(), String> {
  let hook_path = home.join(CLAUDE_HOOK_REL_PATH);
  if let Some(parent) = hook_path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("Create claude hooks dir failed: {e}"))?;
  }

  let script = r#"#!/bin/bash
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r ".tool_input.file_path // empty")

if [[ "$FILE_PATH" != *"SKILL.md" ]]; then
  exit 0
fi

SKILL_NAME=$(basename "$(dirname "$FILE_PATH")")
SESSION_ID=$(echo "$INPUT" | jq -r ".session_id // empty")
CWD=$(echo "$INPUT" | jq -r ".cwd // empty")
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

mkdir -p ~/my-skills/.logs
LOG_FILE="$HOME/my-skills/.logs/skill-usage.jsonl"
echo "{\"ts\":\"$TIMESTAMP\",\"skill\":\"$SKILL_NAME\",\"session\":\"$SESSION_ID\",\"cwd\":\"$CWD\",\"tool\":\"claude-code\"}" >> "$LOG_FILE"
"#;
  fs::write(&hook_path, script).map_err(|e| format!("Write claude hook failed: {e}"))?;

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&hook_path, fs::Permissions::from_mode(0o755))
      .map_err(|e| format!("Set claude hook permissions failed: {e}"))?;
  }

  let settings_path = home.join(".claude").join("settings.json");
  if let Some(parent) = settings_path.parent() {
    fs::create_dir_all(parent).map_err(|e| format!("Create claude settings dir failed: {e}"))?;
  }
  let mut settings = if settings_path.exists() {
    fs::read_to_string(&settings_path)
      .ok()
      .and_then(|raw| serde_json::from_str::<JsonValue>(&raw).ok())
      .unwrap_or_else(|| json!({}))
  } else {
    json!({})
  };
  if !settings.is_object() {
    settings = json!({});
  }

  let root = settings.as_object_mut().expect("settings object");
  if !root.get("hooks").map(|item| item.is_object()).unwrap_or(false) {
    root.insert("hooks".to_string(), json!({}));
  }
  let hooks = root
    .get_mut("hooks")
    .and_then(JsonValue::as_object_mut)
    .ok_or_else(|| "Invalid claude hooks config".to_string())?;
  if !hooks
    .get("PostToolUse")
    .map(|item| item.is_array())
    .unwrap_or(false)
  {
    hooks.insert("PostToolUse".to_string(), json!([]));
  }
  let post_tool_use = hooks
    .get_mut("PostToolUse")
    .and_then(JsonValue::as_array_mut)
    .ok_or_else(|| "Invalid claude PostToolUse config".to_string())?;

  let hook_command = "bash ~/.claude/hooks/skill-tracker.sh";
  let exists = post_tool_use
    .iter()
    .any(|entry| entry.to_string().contains("skill-tracker.sh"));
  if !exists {
    post_tool_use.push(json!({
      "matcher": "Read",
      "hooks": [
        {
          "type": "command",
          "command": hook_command
        }
      ]
    }));
  }

  let content =
    serde_json::to_string_pretty(&settings).map_err(|e| format!("Serialize settings failed: {e}"))?;
  fs::write(settings_path, format!("{content}\n"))
    .map_err(|e| format!("Write claude settings failed: {e}"))
}

fn remove_skill_target(target_dir: &Path, target_file: &Path) -> Result<(), String> {
  remove_if_exists(target_file)?;
  if target_dir.exists() {
    let mut entries = fs::read_dir(target_dir)
      .map_err(|e| format!("Read target dir failed: {e}"))?;
    if entries.next().is_none() {
      fs::remove_dir_all(target_dir).map_err(|e| format!("Remove target dir failed: {e}"))?;
    }
  }
  Ok(())
}

fn is_skill_enabled_for_tool(
  skill_name: &str,
  tool_id: &str,
  config_skills: Option<&[SkillSyncConfig]>,
) -> bool {
  let Some(config_skills) = config_skills else {
    return true;
  };

  let Some(config) = config_skills.iter().find(|item| item.skill_name == skill_name) else {
    return true;
  };

  config
    .enabled_tools
    .iter()
    .any(|enabled| enabled == tool_id)
}

pub fn apply_setup_with_paths(
  home: &Path,
  skills_root: &Path,
  tool_ids: &[String],
  skill_configs: Option<&[SkillSyncConfig]>,
) -> Result<Vec<ApplyResult>, String> {
  let tools = all_tools(home)?;
  let skills = crate::skills::list_skills(skills_root)?;
  if let Some(configs) = skill_configs {
    write_sync_config(home, configs)?;
  }
  let stored_sync_config = if skill_configs.is_some() {
    skill_configs.map(|items| items.to_vec())
  } else {
    read_sync_config(home)?.map(|cfg| cfg.skills)
  };
  let config_ref = stored_sync_config.as_deref();
  let mut out = Vec::<ApplyResult>::new();

  for tool_id in tool_ids {
    let Some(tool) = tools.iter().find(|item| item.id == tool_id.as_str()) else {
      out.push(ApplyResult {
        tool: tool_id.clone(),
        success: false,
        action: "unknown tool".to_string(),
        sync_mode: "none".to_string(),
        synced_count: 0,
        error: Some("Tool id not supported".to_string()),
      });
      continue;
    };

    if let Err(err) = fs::create_dir_all(&tool.skills_dir) {
      out.push(ApplyResult {
        tool: tool.id.clone(),
        success: false,
        action: "create target skills dir failed".to_string(),
        sync_mode: "none".to_string(),
        synced_count: 0,
        error: Some(format!("{err}")),
      });
      continue;
    }

    let mut synced_count = 0usize;
    let mut removed_count = 0usize;
    let mut sync_mode = "symlink".to_string();
    let mut failure: Option<String> = None;

    for skill in &skills {
      let source = PathBuf::from(&skill.directory).join("SKILL.md");
      let target_dir = tool.skills_dir.join(&skill.name);
      let target_file = target_dir.join("SKILL.md");

      if !is_skill_enabled_for_tool(&skill.name, &tool.id, config_ref) {
        if let Err(err) = remove_skill_target(&target_dir, &target_file) {
          failure = Some(err);
          break;
        }
        removed_count += 1;
        continue;
      }

      if let Err(err) = fs::create_dir_all(&target_dir) {
        failure = Some(format!("Create target dir failed: {err}"));
        break;
      }

      match sync_skill_file(&source, &target_file) {
        Ok(mode) => {
          if mode == "copy" {
            sync_mode = "copy".to_string();
          }
          synced_count += 1;
        }
        Err(err) => {
          failure = Some(err);
          break;
        }
      }
    }

    if let Some(error) = failure {
      out.push(ApplyResult {
        tool: tool.id.clone(),
        success: false,
        action: "sync failed".to_string(),
        sync_mode: if synced_count == 0 {
          "none".to_string()
        } else {
          sync_mode
        },
        synced_count,
        error: Some(error),
      });
      continue;
    }

    let mut action_parts = vec![format!(
      "synced {synced_count} skills to {} (removed {removed_count})",
      tool.skills_dir.to_string_lossy()
    )];

    if let Some(rules_path) = tool.rules_path.as_ref() {
      if let Err(err) = ensure_rules_injected(&tool.id, rules_path) {
        out.push(ApplyResult {
          tool: tool.id.clone(),
          success: false,
          action: "rules injection failed".to_string(),
          sync_mode: if synced_count == 0 {
            "none".to_string()
          } else {
            sync_mode.clone()
          },
          synced_count,
          error: Some(err),
        });
        continue;
      }
      action_parts.push(format!("rules injected into {}", rules_path.to_string_lossy()));
    }

    if tool.id == "claude-code" {
      if let Err(err) = ensure_claude_hook(home) {
        out.push(ApplyResult {
          tool: tool.id.clone(),
          success: false,
          action: "claude hook setup failed".to_string(),
          sync_mode: if synced_count == 0 {
            "none".to_string()
          } else {
            sync_mode.clone()
          },
          synced_count,
          error: Some(err),
        });
        continue;
      }
      action_parts.push("claude hook configured".to_string());
    }

    out.push(ApplyResult {
      tool: tool.id.clone(),
      success: true,
      action: action_parts.join("; "),
      sync_mode: if synced_count == 0 {
        "none".to_string()
      } else {
        sync_mode
      },
      synced_count,
      error: None,
    });
  }

  Ok(out)
}

#[tauri::command]
pub fn setup_apply(
  tools: Vec<String>,
  skills: Option<Vec<SkillSyncConfig>>,
) -> Result<Vec<ApplyResult>, String> {
  let home = default_home_dir();
  let skills_root = default_skills_root(&home);
  apply_setup_with_paths(&home, &skills_root, &tools, skills.as_deref())
}

#[cfg(test)]
mod tests {
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
    list.iter().find(|item| item.id == id).expect("find tool status")
  }

  #[test]
  fn custom_tool_registry_adds_and_removes_tool() {
    let home = temp_home();
    let custom = CustomTool {
      name: "Aider".to_string(),
      id: "aider".to_string(),
      skills_dir: home.join(".aider").join("skills").to_string_lossy().to_string(),
      rules_file: Some(
        home
          .join(".aider")
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
    fs::write(codex_skills.join("a.md"), "a").expect("write skill a");
    fs::write(codex_skills.join("b.md"), "b").expect("write skill b");

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

    assert!(
      home
        .join(".codex")
        .join("skills")
        .join("code-review")
        .join("SKILL.md")
        .exists()
    );
    assert!(
      home
        .join(".codex")
        .join("skills")
        .join("debug-helper")
        .join("SKILL.md")
        .exists()
    );
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

    fs::create_dir_all(skills_root.join("debug-helper")).expect("create debug-helper skill dir");
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
    assert!(
      home
        .join(".codex")
        .join("skills")
        .join("code-review")
        .join("SKILL.md")
        .exists()
    );
    assert!(
      !home
        .join(".codex")
        .join("skills")
        .join("debug-helper")
        .join("SKILL.md")
        .exists()
    );
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
    assert!(
      custom_skills_dir
        .join("debug-helper")
        .join("SKILL.md")
        .exists()
    );
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

    let rules_content = fs::read_to_string(home.join(".codex").join("AGENTS.md"))
      .expect("read codex rules");
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

    let result = apply_setup_with_paths(&home, &skills_root, &["claude-code".to_string()], None)
      .expect("apply setup");
    assert!(result[0].success);

    let hook_script = home.join(".claude").join("hooks").join("skill-tracker.sh");
    assert!(hook_script.exists());

    let settings = fs::read_to_string(home.join(".claude").join("settings.json"))
      .expect("read claude settings");
    assert!(settings.contains("skill-tracker.sh"));
  }
}
