use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(super) struct ToolDescriptor {
    pub name: String,
    pub id: String,
    pub icon: Option<String>,
    pub skills_dir: PathBuf,
    pub rules_path: Option<PathBuf>,
    pub path_source: String,
    pub is_custom: bool,
}

#[derive(Debug, Clone)]
struct BuiltInToolDefaults {
    name: &'static str,
    id: &'static str,
    skills_dir: PathBuf,
    rules_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct ToolPathCandidate {
    skills_dir: PathBuf,
    rules_path: Option<PathBuf>,
}

pub(super) fn is_built_in_tool_id(id: &str) -> bool {
    matches!(
        id,
        "antigravity" | "codex" | "claude-code" | "cursor" | "windsurf" | "trae" | "opencode"
    )
}

fn built_in_defaults(home: &Path) -> Vec<BuiltInToolDefaults> {
    vec![
        BuiltInToolDefaults {
            name: "Antigravity",
            id: "antigravity",
            skills_dir: home.join(".gemini").join("instructions"),
            rules_path: Some(home.join(".gemini").join("GEMINI.md")),
        },
        BuiltInToolDefaults {
            name: "Codex",
            id: "codex",
            skills_dir: home.join(".codex").join("skills"),
            rules_path: Some(home.join(".codex").join("AGENTS.md")),
        },
        BuiltInToolDefaults {
            name: "Claude Code",
            id: "claude-code",
            skills_dir: home.join(".claude").join("skills"),
            rules_path: Some(home.join(".claude").join("CLAUDE.md")),
        },
        BuiltInToolDefaults {
            name: "Cursor",
            id: "cursor",
            skills_dir: home.join(".cursor").join("rules"),
            rules_path: Some(
                home.join(".cursor")
                    .join("rules")
                    .join("myskills-tracker.mdc"),
            ),
        },
        BuiltInToolDefaults {
            name: "Windsurf",
            id: "windsurf",
            skills_dir: home.join(".codeium").join("windsurf").join("skills"),
            rules_path: Some(
                home.join(".codeium")
                    .join("windsurf")
                    .join("memories")
                    .join("global_rules.md"),
            ),
        },
        BuiltInToolDefaults {
            name: "Trae",
            id: "trae",
            skills_dir: home.join(".trae").join("skills"),
            rules_path: Some(home.join(".trae").join("AGENTS.md")),
        },
        BuiltInToolDefaults {
            name: "OpenCode",
            id: "opencode",
            skills_dir: home.join(".config").join("opencode").join("skills"),
            rules_path: Some(home.join(".config").join("opencode").join("AGENTS.md")),
        },
    ]
}

fn candidate_paths_for_tool(home: &Path, defaults: &BuiltInToolDefaults) -> Vec<ToolPathCandidate> {
    let mut candidates = vec![ToolPathCandidate {
        skills_dir: defaults.skills_dir.clone(),
        rules_path: defaults.rules_path.clone(),
    }];

    match defaults.id {
        "opencode" => {
            candidates.push(ToolPathCandidate {
                skills_dir: home.join(".opencode").join("skills"),
                rules_path: Some(home.join(".opencode").join("AGENTS.md")),
            });
        }
        "cursor" => {
            candidates.push(ToolPathCandidate {
                skills_dir: home.join(".cursor").join("skills"),
                rules_path: defaults.rules_path.clone(),
            });
        }
        "windsurf" => {
            candidates.push(ToolPathCandidate {
                skills_dir: home.join(".windsurf").join("skills"),
                rules_path: Some(home.join(".windsurf").join("global_rules.md")),
            });
        }
        _ => {}
    }

    candidates
}

pub(super) fn built_in_tools(
    home: &Path,
    overrides: &[super::ToolPathOverride],
) -> Vec<ToolDescriptor> {
    let mut resolved = Vec::<ToolDescriptor>::new();

    for defaults in built_in_defaults(home) {
        if let Some(override_item) = overrides.iter().find(|item| item.id == defaults.id) {
            let skills_dir = override_item.skills_dir.trim();
            if !skills_dir.is_empty() {
                let rules_path = override_item
                    .rules_file
                    .as_ref()
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .map(PathBuf::from);
                resolved.push(ToolDescriptor {
                    name: defaults.name.to_string(),
                    id: defaults.id.to_string(),
                    icon: None,
                    skills_dir: PathBuf::from(skills_dir),
                    rules_path,
                    path_source: "override".to_string(),
                    is_custom: false,
                });
                continue;
            }
        }

        let candidates = candidate_paths_for_tool(home, &defaults);
        let selected = candidates
            .iter()
            .find(|candidate| candidate.skills_dir.exists())
            .cloned();

        let (skills_dir, rules_path, path_source) = if let Some(candidate) = selected {
            let source = if candidate.skills_dir == defaults.skills_dir {
                "default"
            } else {
                "auto-detected"
            };
            (candidate.skills_dir, candidate.rules_path, source.to_string())
        } else {
            (
                defaults.skills_dir.clone(),
                defaults.rules_path.clone(),
                "default".to_string(),
            )
        };

        resolved.push(ToolDescriptor {
            name: defaults.name.to_string(),
            id: defaults.id.to_string(),
            icon: None,
            skills_dir,
            rules_path,
            path_source,
            is_custom: false,
        });
    }

    resolved
}

pub(super) fn custom_tool_to_descriptor(custom: super::CustomTool) -> ToolDescriptor {
    ToolDescriptor {
        name: custom.name,
        id: custom.id,
        icon: custom.icon,
        skills_dir: PathBuf::from(custom.skills_dir),
        rules_path: custom.rules_file.map(PathBuf::from),
        path_source: "custom".to_string(),
        is_custom: true,
    }
}
