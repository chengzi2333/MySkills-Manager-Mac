use std::fs;
use std::path::{Path, PathBuf};

fn custom_tools_file(home: &Path) -> PathBuf {
    crate::root_dir::app_config_dir(home).join("custom-tools.json")
}

fn tool_path_overrides_file(home: &Path) -> PathBuf {
    crate::root_dir::app_config_dir(home).join("tool-path-overrides.json")
}

fn sync_config_file(home: &Path) -> PathBuf {
    crate::root_dir::app_config_dir(home).join("sync-config.json")
}

pub(super) fn read_custom_tools(home: &Path) -> Result<Vec<super::CustomTool>, String> {
    let path = custom_tools_file(home);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("Read custom tools failed: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<super::CustomTool>>(&raw)
        .map_err(|e| format!("Invalid custom tools config: {e}"))
}

pub(super) fn write_custom_tools(home: &Path, tools: &[super::CustomTool]) -> Result<(), String> {
    fs::create_dir_all(crate::root_dir::app_config_dir(home))
        .map_err(|e| format!("Create app config dir failed: {e}"))?;
    let content =
        serde_json::to_string_pretty(tools).map_err(|e| format!("Serialize custom tools failed: {e}"))?;
    fs::write(custom_tools_file(home), format!("{content}\n"))
        .map_err(|e| format!("Write custom tools failed: {e}"))
}

pub(super) fn read_tool_path_overrides(
    home: &Path,
) -> Result<Vec<super::ToolPathOverride>, String> {
    let path = tool_path_overrides_file(home);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let raw =
        fs::read_to_string(path).map_err(|e| format!("Read tool path overrides failed: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }

    serde_json::from_str::<Vec<super::ToolPathOverride>>(&raw)
        .map_err(|e| format!("Invalid tool path overrides config: {e}"))
}

pub(super) fn write_tool_path_overrides(
    home: &Path,
    overrides: &[super::ToolPathOverride],
) -> Result<(), String> {
    fs::create_dir_all(crate::root_dir::app_config_dir(home))
        .map_err(|e| format!("Create app config dir failed: {e}"))?;
    let content = serde_json::to_string_pretty(overrides)
        .map_err(|e| format!("Serialize tool path overrides failed: {e}"))?;
    fs::write(tool_path_overrides_file(home), format!("{content}\n"))
        .map_err(|e| format!("Write tool path overrides failed: {e}"))
}

pub(super) fn read_sync_config(home: &Path) -> Result<Option<super::SyncConfigFile>, String> {
    let path = sync_config_file(home);
    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(path).map_err(|e| format!("Read sync config failed: {e}"))?;
    if raw.trim().is_empty() {
        return Ok(None);
    }

    let mut parsed =
        serde_json::from_str::<super::SyncConfigFile>(&raw).map_err(|e| format!("Invalid sync config: {e}"))?;
    parsed.auto_tools = super::normalize_tool_ids(parsed.auto_tools);
    parsed.tracking_disabled_tools = super::normalize_tool_ids(parsed.tracking_disabled_tools);
    Ok(Some(parsed))
}

pub(super) fn write_sync_config_file(
    home: &Path,
    config: &super::SyncConfigFile,
) -> Result<(), String> {
    fs::create_dir_all(crate::root_dir::app_config_dir(home))
        .map_err(|e| format!("Create app config dir failed: {e}"))?;
    let content =
        serde_json::to_string_pretty(config).map_err(|e| format!("Serialize sync config failed: {e}"))?;
    fs::write(sync_config_file(home), format!("{content}\n"))
        .map_err(|e| format!("Write sync config failed: {e}"))
}
