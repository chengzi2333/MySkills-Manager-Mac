use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use config_store::{
    read_custom_tools, read_sync_config, read_tool_path_overrides, write_custom_tools,
    write_sync_config_file, write_tool_path_overrides,
};
use rule_hook_ops::{
    ensure_claude_hook, ensure_claude_hook_removed, ensure_rules_injected, ensure_rules_removed,
};
use status_probe::{detect_claude_hook, detect_sync_stats, file_contains_marker};
use tool_catalog::{
    built_in_tools, custom_tool_to_descriptor, is_built_in_tool_id, ToolDescriptor,
};

mod config_store;
mod apply_engine;
mod rule_hook_ops;
mod sync_ops;
mod status_probe;
mod tool_catalog;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolStatus {
    pub name: String,
    pub id: String,
    pub icon: Option<String>,
    pub skills_dir: String,
    pub rules_path: String,
    pub path_source: String,
    pub exists: bool,
    pub configured: bool,
    pub synced_skills: usize,
    pub sync_mode: String,
    pub last_sync_time: Option<String>,
    pub auto_sync: bool,
    pub tracking_enabled: bool,
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillOverviewEntry {
    pub name: String,
    pub content_hash: String,
    pub duplicate_across_tools: bool,
    pub in_my_skills: bool,
    pub hash_matches_my_skills: bool,
    pub hash_conflicts_my_skills: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolSkillOverview {
    pub tool_id: String,
    pub tool_name: String,
    pub skills: Vec<SkillOverviewEntry>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillsOverview {
    pub tools: Vec<ToolSkillOverview>,
    pub duplicate_names: Vec<String>,
    pub total_skills: usize,
    pub unique_skills: usize,
    pub matched_in_my_skills: usize,
    pub missing_in_my_skills: usize,
    pub conflict_with_my_skills: usize,
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
    #[serde(default)]
    auto_tools: Vec<String>,
    #[serde(default)]
    tracking_disabled_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ToolPathOverride {
    id: String,
    skills_dir: String,
    rules_file: Option<String>,
}

#[derive(Debug, Clone)]
struct RollbackPath {
    path: PathBuf,
    existed: bool,
}

const TRACKER_BLOCK_START: &str =
    "<!-- [MySkills Manager] Skill usage tracking rule - DO NOT REMOVE -->";
const TRACKER_BLOCK_END: &str = "<!-- [/MySkills Manager] -->";
const CLAUDE_HOOK_REL_PATH: &str = ".claude/hooks/skill-tracker.sh";

fn default_sync_mode() -> String {
    "symlink".to_string()
}

fn normalize_tool_ids(ids: Vec<String>) -> Vec<String> {
    let mut out = ids
        .into_iter()
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>();
    out.sort();
    out.dedup();
    out
}

fn write_sync_config(home: &Path, skills: &[SkillSyncConfig]) -> Result<(), String> {
    let existing = read_sync_config(home)?;

    let sync_mode = existing
        .as_ref()
        .map(|cfg| cfg.sync_mode.clone())
        .unwrap_or_else(default_sync_mode);
    let auto_tools = existing
        .as_ref()
        .map(|cfg| cfg.auto_tools.clone())
        .unwrap_or_default();
    let tracking_disabled_tools = existing
        .map(|cfg| cfg.tracking_disabled_tools)
        .unwrap_or_default();
    write_sync_config_file(
        home,
        &SyncConfigFile {
            sync_mode,
            skills: skills.to_vec(),
            auto_tools,
            tracking_disabled_tools,
        },
    )
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

fn all_tools(home: &Path) -> Result<Vec<ToolDescriptor>, String> {
    let overrides = read_tool_path_overrides(home)?;
    let mut tools = built_in_tools(home, &overrides);
    for custom in read_custom_tools(home)? {
        tools.push(custom_tool_to_descriptor(custom));
    }
    Ok(tools)
}

fn tool_skill_source_dirs(tool: &ToolDescriptor) -> Vec<PathBuf> {
    vec![tool.skills_dir.clone(), tool.skills_dir.join(".system")]
}

pub fn setup_skill_source_dirs_with_home(
    home: &Path,
) -> Result<Vec<(String, String, Vec<PathBuf>)>, String> {
    let mut out = Vec::<(String, String, Vec<PathBuf>)>::new();
    for tool in all_tools(home)? {
        let sources = tool_skill_source_dirs(&tool);
        out.push((tool.id, tool.name, sources));
    }
    Ok(out)
}

fn skill_file_hash(skill_file: &Path) -> Result<String, String> {
    let raw = fs::read(skill_file).map_err(|e| format!("Read SKILL.md failed: {e}"))?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw.hash(&mut hasher);
    Ok(format!("{:016x}", hasher.finish()))
}

fn skill_hashes_by_name(root: &Path) -> Result<HashMap<String, String>, String> {
    let mut out = HashMap::<String, String>::new();
    if !root.exists() {
        return Ok(out);
    }

    let skills = crate::skills::list_skills(root)?;
    for skill in skills {
        let skill_path = PathBuf::from(&skill.directory).join("SKILL.md");
        let hash = skill_file_hash(&skill_path)?;
        out.insert(skill.name, hash);
    }

    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NameSyncState {
    Missing,
    Matched,
    Conflict,
}

pub fn local_skills_overview_with_home(home: &Path) -> Result<LocalSkillsOverview, String> {
    let sources = setup_skill_source_dirs_with_home(home)?;
    let my_skill_hashes = skill_hashes_by_name(&crate::root_dir::default_skills_root(home))?;

    let mut raw_tools = Vec::<(String, String, Vec<(String, String)>)>::new();
    let mut name_to_tools = HashMap::<String, usize>::new();
    let mut name_to_hashes = HashMap::<String, HashSet<String>>::new();
    let mut total_skills = 0usize;

    for (tool_id, tool_name, source_dirs) in sources {
        let mut seen = HashMap::<String, String>::new();
        for source_dir in source_dirs {
            if !source_dir.exists() {
                continue;
            }
            let skills = crate::skills::list_skills(&source_dir)?;
            for skill in skills {
                if seen.contains_key(&skill.name) {
                    continue;
                }
                let skill_path = PathBuf::from(&skill.directory).join("SKILL.md");
                let hash = skill_file_hash(&skill_path)?;
                seen.insert(skill.name, hash);
            }
        }

        if seen.is_empty() {
            continue;
        }

        let mut named_hashes = seen.into_iter().collect::<Vec<_>>();
        named_hashes.sort_by(|a, b| a.0.cmp(&b.0));
        total_skills += named_hashes.len();
        for (name, hash) in &named_hashes {
            *name_to_tools.entry(name.clone()).or_insert(0) += 1;
            name_to_hashes
                .entry(name.clone())
                .or_default()
                .insert(hash.clone());
        }

        raw_tools.push((tool_id, tool_name, named_hashes));
    }

    let mut duplicate_names = name_to_tools
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    duplicate_names.sort();
    let duplicate_name_set = duplicate_names.iter().cloned().collect::<HashSet<_>>();

    let mut name_states = HashMap::<String, NameSyncState>::new();
    for (name, hashes) in &name_to_hashes {
        let state = match my_skill_hashes.get(name) {
            None => NameSyncState::Missing,
            Some(my_hash) => {
                if hashes.iter().all(|hash| hash == my_hash) {
                    NameSyncState::Matched
                } else {
                    NameSyncState::Conflict
                }
            }
        };
        name_states.insert(name.clone(), state);
    }

    let matched_in_my_skills = name_states
        .values()
        .filter(|state| **state == NameSyncState::Matched)
        .count();
    let missing_in_my_skills = name_states
        .values()
        .filter(|state| **state == NameSyncState::Missing)
        .count();
    let conflict_with_my_skills = name_states
        .values()
        .filter(|state| **state == NameSyncState::Conflict)
        .count();

    let mut tools = Vec::<ToolSkillOverview>::new();
    for (tool_id, tool_name, named_hashes) in raw_tools {
        let skills = named_hashes
            .into_iter()
            .map(|(name, content_hash)| {
                let my_hash = my_skill_hashes.get(&name);
                let in_my_skills = my_hash.is_some();
                let hash_matches_my_skills =
                    my_hash.map(|hash| hash == &content_hash).unwrap_or(false);

                SkillOverviewEntry {
                    duplicate_across_tools: duplicate_name_set.contains(&name),
                    hash_conflicts_my_skills: in_my_skills && !hash_matches_my_skills,
                    hash_matches_my_skills,
                    in_my_skills,
                    name,
                    content_hash,
                }
            })
            .collect::<Vec<_>>();

        tools.push(ToolSkillOverview {
            count: skills.len(),
            skills,
            tool_id,
            tool_name,
        });
    }

    tools.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));

    Ok(LocalSkillsOverview {
        conflict_with_my_skills,
        matched_in_my_skills,
        missing_in_my_skills,
        unique_skills: name_to_tools.len(),
        tools,
        duplicate_names,
        total_skills,
    })
}

pub fn setup_status_with_home(home: &Path) -> Result<Vec<ToolStatus>, String> {
    let sync_config = read_sync_config(home)?;
    let auto_tools = sync_config
        .as_ref()
        .map(|cfg| cfg.auto_tools.iter().cloned().collect::<HashSet<_>>())
        .unwrap_or_default();
    let tracking_disabled_tools = sync_config
        .map(|cfg| {
            cfg.tracking_disabled_tools
                .into_iter()
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
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
        let (synced_skills, sync_mode, last_sync_time) = detect_sync_stats(&tool.skills_dir)?;
        let hook_configured = if tool.id == "claude-code" {
            detect_claude_hook(home)
        } else {
            false
        };

        list.push(ToolStatus {
            name: tool.name.clone(),
            id: tool.id.clone(),
            icon: tool.icon.clone(),
            skills_dir: tool.skills_dir.to_string_lossy().to_string(),
            rules_path: tool
                .rules_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string())
                .unwrap_or_default(),
            path_source: tool.path_source.clone(),
            exists,
            configured,
            synced_skills,
            sync_mode,
            last_sync_time,
            auto_sync: auto_tools.contains(&tool.id),
            tracking_enabled: !tracking_disabled_tools.contains(&tool.id),
            hook_configured,
            is_custom: tool.is_custom,
        });
    }
    Ok(list)
}

#[tauri::command]
pub fn setup_status() -> Result<Vec<ToolStatus>, String> {
    setup_status_with_home(&crate::root_dir::default_home_dir())
}

#[tauri::command]
pub fn setup_local_skills_overview() -> Result<LocalSkillsOverview, String> {
    local_skills_overview_with_home(&crate::root_dir::default_home_dir())
}

pub fn get_custom_tools_with_home(home: &Path) -> Result<Vec<CustomTool>, String> {
    read_custom_tools(home)
}

pub fn add_custom_tool_with_home(
    home: &Path,
    mut tool: CustomTool,
) -> Result<SetupMutationResult, String> {
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
    if is_built_in_tool_id(&tool.id) {
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

pub fn update_tool_paths_with_home(
    home: &Path,
    id: &str,
    skills_dir: &str,
    rules_file: Option<&str>,
) -> Result<SetupMutationResult, String> {
    let tool_id = validate_tool_id(id)?;
    let normalized_skills_dir = skills_dir.trim();
    if normalized_skills_dir.is_empty() {
        return Err("skillsDir is required".to_string());
    }
    let normalized_rules_file = rules_file
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut custom_tools = read_custom_tools(home)?;
    if let Some(custom_tool) = custom_tools.iter_mut().find(|item| item.id == tool_id) {
        custom_tool.skills_dir = normalized_skills_dir.to_string();
        custom_tool.rules_file = normalized_rules_file;
        write_custom_tools(home, &custom_tools)?;

        let mut overrides = read_tool_path_overrides(home)?;
        let before = overrides.len();
        overrides.retain(|item| item.id != tool_id);
        if overrides.len() != before {
            write_tool_path_overrides(home, &overrides)?;
        }

        return Ok(SetupMutationResult { success: true });
    }

    if is_built_in_tool_id(&tool_id) {
        let mut overrides = read_tool_path_overrides(home)?;
        if let Some(existing) = overrides.iter_mut().find(|item| item.id == tool_id) {
            existing.skills_dir = normalized_skills_dir.to_string();
            existing.rules_file = normalized_rules_file;
        } else {
            overrides.push(ToolPathOverride {
                id: tool_id.clone(),
                skills_dir: normalized_skills_dir.to_string(),
                rules_file: normalized_rules_file,
            });
        }
        overrides.sort_by(|a, b| a.id.cmp(&b.id));
        write_tool_path_overrides(home, &overrides)?;
        return Ok(SetupMutationResult { success: true });
    }

    Err("Tool id not supported".to_string())
}

pub fn set_tool_auto_sync_with_home(
    home: &Path,
    id: &str,
    enabled: bool,
) -> Result<SetupMutationResult, String> {
    let tool_id = validate_tool_id(id)?;
    let supported = all_tools(home)?.iter().any(|tool| tool.id == tool_id);
    if !supported {
        return Err("Tool id not supported".to_string());
    }

    let mut config = read_sync_config(home)?.unwrap_or(SyncConfigFile {
        sync_mode: default_sync_mode(),
        skills: Vec::new(),
        auto_tools: Vec::new(),
        tracking_disabled_tools: Vec::new(),
    });

    let mut auto_tools = config.auto_tools.into_iter().collect::<HashSet<_>>();
    if enabled {
        auto_tools.insert(tool_id.clone());
    } else {
        auto_tools.remove(tool_id.as_str());
    }

    config.auto_tools = normalize_tool_ids(auto_tools.into_iter().collect());
    write_sync_config_file(home, &config)?;
    Ok(SetupMutationResult { success: true })
}

pub fn set_tool_tracking_enabled_with_home(
    home: &Path,
    id: &str,
    enabled: bool,
) -> Result<SetupMutationResult, String> {
    let tool_id = validate_tool_id(id)?;
    let tools = all_tools(home)?;
    let Some(tool) = tools.iter().find(|item| item.id == tool_id) else {
        return Err("Tool id not supported".to_string());
    };

    let mut config = read_sync_config(home)?.unwrap_or(SyncConfigFile {
        sync_mode: default_sync_mode(),
        skills: Vec::new(),
        auto_tools: Vec::new(),
        tracking_disabled_tools: Vec::new(),
    });

    let mut disabled_tools = config
        .tracking_disabled_tools
        .into_iter()
        .collect::<HashSet<_>>();
    if enabled {
        disabled_tools.remove(tool_id.as_str());
    } else {
        disabled_tools.insert(tool_id.clone());
    }

    config.tracking_disabled_tools = normalize_tool_ids(disabled_tools.into_iter().collect());
    write_sync_config_file(home, &config)?;

    if let Some(rules_path) = tool.rules_path.as_ref() {
        if enabled {
            ensure_rules_injected(&tool_id, rules_path)?;
        } else {
            ensure_rules_removed(rules_path)?;
        }
    }

    if tool_id == "claude-code" {
        if enabled {
            ensure_claude_hook(home)?;
        } else {
            ensure_claude_hook_removed(home)?;
        }
    }

    Ok(SetupMutationResult { success: true })
}

#[tauri::command]
pub fn setup_get_custom_tools() -> Result<Vec<CustomTool>, String> {
    get_custom_tools_with_home(&crate::root_dir::default_home_dir())
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
        &crate::root_dir::default_home_dir(),
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
    remove_custom_tool_with_home(&crate::root_dir::default_home_dir(), &id)
}

#[tauri::command]
pub fn setup_update_tool_paths(
    id: String,
    skills_dir: String,
    rules_file: Option<String>,
) -> Result<SetupMutationResult, String> {
    update_tool_paths_with_home(
        &crate::root_dir::default_home_dir(),
        &id,
        &skills_dir,
        rules_file.as_deref(),
    )
}

#[tauri::command]
pub fn setup_set_tool_auto_sync(id: String, enabled: bool) -> Result<SetupMutationResult, String> {
    set_tool_auto_sync_with_home(&crate::root_dir::default_home_dir(), &id, enabled)
}

#[tauri::command]
pub fn setup_set_tool_tracking_enabled(
    id: String,
    enabled: bool,
) -> Result<SetupMutationResult, String> {
    set_tool_tracking_enabled_with_home(&crate::root_dir::default_home_dir(), &id, enabled)
}

pub fn sync_saved_skill_to_copy_tools_with_home(
    home: &Path,
    skills_root: &Path,
    skill_name: &str,
) -> Result<usize, String> {
    apply_engine::sync_saved_skill_to_copy_tools_with_home(home, skills_root, skill_name)
}

pub fn apply_setup_with_paths(
    home: &Path,
    skills_root: &Path,
    tool_ids: &[String],
    skill_configs: Option<&[SkillSyncConfig]>,
) -> Result<Vec<ApplyResult>, String> {
    apply_engine::apply_setup_with_paths(home, skills_root, tool_ids, skill_configs)
}

#[tauri::command]
pub fn setup_apply(
    tools: Vec<String>,
    skills: Option<Vec<SkillSyncConfig>>,
) -> Result<Vec<ApplyResult>, String> {
    let home = crate::root_dir::default_home_dir();
    let skills_root = crate::root_dir::default_skills_root(&home);
    apply_setup_with_paths(&home, &skills_root, &tools, skills.as_deref())
}

#[cfg(test)]
mod tests;

