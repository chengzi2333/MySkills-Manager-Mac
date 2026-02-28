use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn is_skill_enabled_for_tool(
    skill_name: &str,
    tool_id: &str,
    config_skills: Option<&[super::SkillSyncConfig]>,
) -> bool {
    let Some(config_skills) = config_skills else {
        return true;
    };

    let Some(config) = config_skills
        .iter()
        .find(|item| item.skill_name == skill_name)
    else {
        return false;
    };

    config.enabled_tools.iter().any(|enabled| enabled == tool_id)
}

fn is_tracking_enabled_for_tool(tool_id: &str, tracking_disabled_tools: &HashSet<String>) -> bool {
    !tracking_disabled_tools.contains(tool_id)
}

pub(super) fn sync_saved_skill_to_copy_tools_with_home(
    home: &Path,
    skills_root: &Path,
    skill_name: &str,
) -> Result<usize, String> {
    let tools = super::all_tools(home)?;
    let stored_sync_config = super::read_sync_config(home)?;
    let config_ref = stored_sync_config.as_ref().map(|cfg| cfg.skills.as_slice());

    let source_file = crate::skills::list_skills(skills_root)?
        .into_iter()
        .find(|item| item.name == skill_name)
        .map(|item| PathBuf::from(item.directory).join("SKILL.md"))
        .unwrap_or_else(|| skills_root.join(skill_name).join("SKILL.md"));

    if !source_file.exists() {
        return Ok(0);
    }

    let mut synced = 0usize;
    for tool in tools {
        if !tool
            .skills_dir
            .parent()
            .map(|parent| parent.exists())
            .unwrap_or(false)
        {
            continue;
        }
        if !is_skill_enabled_for_tool(skill_name, &tool.id, config_ref) {
            continue;
        }

        let (_, mode, _) = super::detect_sync_stats(&tool.skills_dir)?;
        if mode != "copy" {
            continue;
        }

        let target_dir = tool.skills_dir.join(skill_name);
        let target_file = target_dir.join("SKILL.md");
        fs::create_dir_all(&target_dir).map_err(|e| format!("Create target dir failed: {e}"))?;
        super::sync_ops::remove_if_exists(&target_file)?;
        fs::copy(&source_file, &target_file).map_err(|e| format!("Copy skill file failed: {e}"))?;
        synced += 1;
    }

    Ok(synced)
}

pub(super) fn apply_setup_with_paths(
    home: &Path,
    skills_root: &Path,
    tool_ids: &[String],
    skill_configs: Option<&[super::SkillSyncConfig]>,
) -> Result<Vec<super::ApplyResult>, String> {
    let tools = super::all_tools(home)?;
    let skills = crate::skills::list_skills(skills_root)?;
    if let Some(configs) = skill_configs {
        super::write_sync_config(home, configs)?;
    }
    let stored_sync_config = super::read_sync_config(home)?;
    let config_ref = stored_sync_config.as_ref().map(|cfg| cfg.skills.as_slice());
    let tracking_disabled_tools = stored_sync_config
        .as_ref()
        .map(|cfg| {
            cfg.tracking_disabled_tools
                .iter()
                .cloned()
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let mut out = Vec::<super::ApplyResult>::new();
    let mut rollback_paths = Vec::<super::RollbackPath>::new();

    for tool_id in tool_ids {
        let Some(tool) = tools.iter().find(|item| item.id == tool_id.as_str()) else {
            return Ok(super::sync_ops::finalize_with_rollback(
                out,
                super::ApplyResult {
                    tool: tool_id.clone(),
                    success: false,
                    action: "unknown tool".to_string(),
                    sync_mode: "none".to_string(),
                    synced_count: 0,
                    error: Some("Tool id not supported".to_string()),
                },
                &rollback_paths,
            ));
        };

        if let Err(err) = fs::create_dir_all(&tool.skills_dir) {
            return Ok(super::sync_ops::finalize_with_rollback(
                out,
                super::ApplyResult {
                    tool: tool.id.clone(),
                    success: false,
                    action: "create target skills dir failed".to_string(),
                    sync_mode: "none".to_string(),
                    synced_count: 0,
                    error: Some(format!("{err}")),
                },
                &rollback_paths,
            ));
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
                if let Err(err) = super::sync_ops::remove_skill_target(&target_dir, &target_file) {
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

            match super::sync_ops::sync_skill_file(&source, &target_file) {
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
            let failure_result = super::ApplyResult {
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
            };
            return Ok(super::sync_ops::finalize_with_rollback(
                out,
                failure_result,
                &rollback_paths,
            ));
        }

        let mut action_parts = vec![format!(
            "synced {synced_count} skills to {} (removed {removed_count})",
            tool.skills_dir.to_string_lossy()
        )];

        let tracking_enabled = is_tracking_enabled_for_tool(&tool.id, &tracking_disabled_tools);
        if let Some(rules_path) = tool.rules_path.as_ref() {
            super::sync_ops::register_rollback_path(&mut rollback_paths, rules_path);
            let rules_result = if tracking_enabled {
                super::rule_hook_ops::ensure_rules_injected(&tool.id, rules_path)
            } else {
                super::rule_hook_ops::ensure_rules_removed(rules_path)
            };

            if let Err(err) = rules_result {
                let failure_result = super::ApplyResult {
                    tool: tool.id.clone(),
                    success: false,
                    action: "rules sync failed".to_string(),
                    sync_mode: if synced_count == 0 {
                        "none".to_string()
                    } else {
                        sync_mode.clone()
                    },
                    synced_count,
                    error: Some(err),
                };
                return Ok(super::sync_ops::finalize_with_rollback(
                    out,
                    failure_result,
                    &rollback_paths,
                ));
            }
            action_parts.push(if tracking_enabled {
                format!("rules injected into {}", rules_path.to_string_lossy())
            } else {
                format!("rules removed from {}", rules_path.to_string_lossy())
            });
        }

        if tool.id == "claude-code" {
            super::sync_ops::register_rollback_path(
                &mut rollback_paths,
                &home.join(super::CLAUDE_HOOK_REL_PATH),
            );
            super::sync_ops::register_rollback_path(
                &mut rollback_paths,
                &home.join(".claude").join("settings.json"),
            );
            let hook_result = if tracking_enabled {
                super::rule_hook_ops::ensure_claude_hook(home)
            } else {
                super::rule_hook_ops::ensure_claude_hook_removed(home)
            };
            if let Err(err) = hook_result {
                let failure_result = super::ApplyResult {
                    tool: tool.id.clone(),
                    success: false,
                    action: "claude hook sync failed".to_string(),
                    sync_mode: if synced_count == 0 {
                        "none".to_string()
                    } else {
                        sync_mode.clone()
                    },
                    synced_count,
                    error: Some(err),
                };
                return Ok(super::sync_ops::finalize_with_rollback(
                    out,
                    failure_result,
                    &rollback_paths,
                ));
            }
            action_parts.push(if tracking_enabled {
                "claude hook configured".to_string()
            } else {
                "claude hook removed".to_string()
            });
        }

        out.push(super::ApplyResult {
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
