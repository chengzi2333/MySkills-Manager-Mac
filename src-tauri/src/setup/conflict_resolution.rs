use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use super::{setup_skill_source_dirs_with_home, SetupMutationResult, SkillConflictDetail, SkillConflictVariant};

fn content_hash(raw: &[u8]) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    raw.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn my_skill_file(root: &Path, skill_name: &str) -> Option<PathBuf> {
    crate::skills::list_skills(root)
        .ok()
        .and_then(|skills| {
            skills
                .into_iter()
                .find(|skill| skill.name == skill_name)
                .map(|skill| PathBuf::from(skill.directory).join("SKILL.md"))
        })
        .filter(|path| path.exists())
}

fn first_tool_skill_file(source_dirs: &[PathBuf], skill_name: &str) -> Option<PathBuf> {
    for source_dir in source_dirs {
        if !source_dir.exists() {
            continue;
        }
        let Ok(skills) = crate::skills::list_skills(source_dir) else {
            continue;
        };
        if let Some(skill) = skills.into_iter().find(|item| item.name == skill_name) {
            let path = PathBuf::from(skill.directory).join("SKILL.md");
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

pub(super) fn setup_get_skill_conflict_detail_with_home(
    home: &Path,
    skill_name: &str,
) -> Result<SkillConflictDetail, String> {
    let skill_name = skill_name.trim();
    if skill_name.is_empty() {
        return Err("skill name is required".to_string());
    }

    let skills_root = crate::root_dir::default_skills_root(home);
    let mut variants = Vec::<SkillConflictVariant>::new();

    let mut my_hash = None::<String>;
    if let Some(file) = my_skill_file(&skills_root, skill_name) {
        let content = fs::read_to_string(&file).map_err(|e| format!("Read my skill failed: {e}"))?;
        let hash = content_hash(content.as_bytes());
        my_hash = Some(hash.clone());
        variants.push(SkillConflictVariant {
            source_id: "my-skills".to_string(),
            source_name: "My Skills".to_string(),
            content_hash: hash,
            in_my_skills: true,
            hash_matches_my_skills: true,
            content,
        });
    }

    let mut tool_sources = setup_skill_source_dirs_with_home(home)?;
    tool_sources.sort_by(|a, b| a.1.cmp(&b.1));
    for (tool_id, tool_name, source_dirs) in tool_sources {
        if let Some(file) = first_tool_skill_file(&source_dirs, skill_name) {
            let content =
                fs::read_to_string(&file).map_err(|e| format!("Read skill from {tool_name} failed: {e}"))?;
            let hash = content_hash(content.as_bytes());
            variants.push(SkillConflictVariant {
                source_id: tool_id,
                source_name: tool_name,
                content_hash: hash.clone(),
                in_my_skills: my_hash.is_some(),
                hash_matches_my_skills: my_hash.as_ref().map(|value| value == &hash).unwrap_or(false),
                content,
            });
        }
    }

    let mut dedup = HashMap::<String, SkillConflictVariant>::new();
    for variant in variants {
        dedup.entry(variant.source_id.clone()).or_insert(variant);
    }
    let mut variants = dedup.into_values().collect::<Vec<_>>();
    variants.sort_by(|a, b| a.source_name.cmp(&b.source_name));

    Ok(SkillConflictDetail {
        skill_name: skill_name.to_string(),
        variants,
    })
}

pub(super) fn setup_resolve_skill_conflict_with_home(
    home: &Path,
    skill_name: &str,
    source_id: &str,
) -> Result<SetupMutationResult, String> {
    let skill_name = skill_name.trim();
    if skill_name.is_empty() {
        return Err("skill name is required".to_string());
    }
    let source_id = source_id.trim();
    if source_id.is_empty() {
        return Err("source id is required".to_string());
    }

    let detail = setup_get_skill_conflict_detail_with_home(home, skill_name)?;
    let source = detail
        .variants
        .iter()
        .find(|variant| variant.source_id == source_id)
        .ok_or_else(|| format!("source variant not found: {source_id}"))?;

    let skills_root = crate::root_dir::default_skills_root(home);
    let target_dir = skills_root.join(skill_name);
    fs::create_dir_all(&target_dir).map_err(|e| format!("Create target skill dir failed: {e}"))?;
    fs::write(target_dir.join("SKILL.md"), &source.content)
        .map_err(|e| format!("Write resolved SKILL.md failed: {e}"))?;

    crate::setup::sync_saved_skill_to_copy_tools_with_home(home, &skills_root, skill_name)?;
    Ok(SetupMutationResult { success: true })
}
