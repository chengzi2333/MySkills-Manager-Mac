use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::{Mapping, Value as YamlValue};

#[derive(Debug, Serialize, Clone)]
pub struct SkillMeta {
  pub name: String,
  pub description: Option<String>,
  pub category: Option<String>,
  pub tags: Option<Vec<String>>,
  pub my_notes: Option<String>,
  pub last_updated: Option<String>,
  pub directory: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SkillDocument {
  pub frontmatter: JsonValue,
  pub body: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct SaveResult {
  pub success: bool,
}

fn default_root_dir() -> PathBuf {
  if let Ok(path) = std::env::var("MYSKILLS_ROOT_DIR") {
    return PathBuf::from(path);
  }

  if let Ok(home) = std::env::var("HOME") {
    return Path::new(&home).join("my-skills");
  }
  if let Ok(home) = std::env::var("USERPROFILE") {
    return Path::new(&home).join("my-skills");
  }

  PathBuf::from("./")
}

fn split_frontmatter(raw: &str) -> Result<(Mapping, String), String> {
  let normalized = raw.replace("\r\n", "\n");
  if !normalized.starts_with("---\n") {
    return Ok((Mapping::new(), normalized));
  }

  let marker = "\n---\n";
  let rest = &normalized[4..];
  let Some(end) = rest.find(marker) else {
    return Err("Invalid frontmatter block".to_string());
  };

  let yaml_str = &rest[..end];
  let body = rest[end + marker.len()..].to_string();
  let frontmatter = if yaml_str.trim().is_empty() {
    Mapping::new()
  } else {
    serde_yaml::from_str::<Mapping>(yaml_str).map_err(|e| format!("Invalid YAML frontmatter: {e}"))?
  };
  Ok((frontmatter, body))
}

fn build_markdown(frontmatter: &Mapping, body: &str) -> Result<String, String> {
  let yaml = serde_yaml::to_string(frontmatter).map_err(|e| format!("Serialize YAML failed: {e}"))?;
  Ok(format!("---\n{}---\n\n{}", yaml, body.trim_start_matches('\n')))
}

fn yaml_get_string(map: &Mapping, key: &str) -> Option<String> {
  map.get(YamlValue::String(key.to_string()))
    .and_then(|v| v.as_str().map(std::string::ToString::to_string))
}

fn yaml_get_tags(map: &Mapping) -> Option<Vec<String>> {
  map.get(YamlValue::String("tags".to_string()))
    .and_then(|value| value.as_sequence())
    .map(|seq| {
      seq.iter()
        .filter_map(|item| item.as_str().map(std::string::ToString::to_string))
        .collect::<Vec<_>>()
    })
}

fn locate_skill_dir(root: &Path, name: &str) -> Result<PathBuf, String> {
  let entries = fs::read_dir(root).map_err(|e| format!("Read root dir failed: {e}"))?;
  for entry in entries {
    let entry = entry.map_err(|e| format!("Read entry failed: {e}"))?;
    if !entry
      .file_type()
      .map_err(|e| format!("Read file type failed: {e}"))?
      .is_dir()
    {
      continue;
    }
    let skill_file = entry.path().join("SKILL.md");
    if !skill_file.exists() {
      continue;
    }

    let raw = fs::read_to_string(&skill_file).map_err(|e| format!("Read SKILL.md failed: {e}"))?;
    let (frontmatter, _) = split_frontmatter(&raw)?;
    let skill_name = yaml_get_string(&frontmatter, "name").unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
    if skill_name == name || entry.file_name().to_string_lossy() == name {
      return Ok(entry.path());
    }
  }

  Ok(root.join(name))
}

pub fn list_skills(root: &Path) -> Result<Vec<SkillMeta>, String> {
  if !root.exists() {
    return Ok(Vec::new());
  }

  let mut out = Vec::new();
  let entries = fs::read_dir(root).map_err(|e| format!("Read root dir failed: {e}"))?;
  for entry in entries {
    let entry = entry.map_err(|e| format!("Read entry failed: {e}"))?;
    if !entry
      .file_type()
      .map_err(|e| format!("Read file type failed: {e}"))?
      .is_dir()
    {
      continue;
    }

    let file_path = entry.path().join("SKILL.md");
    if !file_path.exists() {
      continue;
    }

    let raw = fs::read_to_string(&file_path).map_err(|e| format!("Read SKILL.md failed: {e}"))?;
    let (frontmatter, _) = split_frontmatter(&raw)?;
    let name = yaml_get_string(&frontmatter, "name").unwrap_or_else(|| entry.file_name().to_string_lossy().to_string());
    out.push(SkillMeta {
      name,
      description: yaml_get_string(&frontmatter, "description"),
      category: yaml_get_string(&frontmatter, "category"),
      tags: yaml_get_tags(&frontmatter),
      my_notes: yaml_get_string(&frontmatter, "my_notes"),
      last_updated: yaml_get_string(&frontmatter, "last_updated"),
      directory: entry.path().to_string_lossy().to_string(),
    });
  }

  out.sort_by(|a, b| a.name.cmp(&b.name));
  Ok(out)
}

pub fn get_content(root: &Path, name: &str) -> Result<SkillDocument, String> {
  let dir = locate_skill_dir(root, name)?;
  let file_path = dir.join("SKILL.md");
  let raw = fs::read_to_string(file_path).map_err(|e| format!("Read SKILL.md failed: {e}"))?;
  let (frontmatter, body) = split_frontmatter(&raw)?;
  let frontmatter_json = serde_json::to_value(frontmatter).map_err(|e| format!("Frontmatter conversion failed: {e}"))?;
  Ok(SkillDocument {
    frontmatter: frontmatter_json,
    body,
  })
}

pub fn save_content(root: &Path, name: &str, content: &str, today: &str) -> Result<SaveResult, String> {
  let dir = locate_skill_dir(root, name)?;
  fs::create_dir_all(&dir).map_err(|e| format!("Create skill dir failed: {e}"))?;
  let file_path = dir.join("SKILL.md");

  let (mut frontmatter, body) = split_frontmatter(content)?;
  frontmatter.insert(
    YamlValue::String("last_updated".to_string()),
    YamlValue::String(today.to_string()),
  );
  let next = build_markdown(&frontmatter, &body)?;
  fs::write(file_path, next).map_err(|e| format!("Write SKILL.md failed: {e}"))?;
  Ok(SaveResult { success: true })
}

#[tauri::command]
pub fn skills_list() -> Result<Vec<SkillMeta>, String> {
  list_skills(&default_root_dir())
}

#[tauri::command]
pub fn skills_get_content(name: String) -> Result<SkillDocument, String> {
  get_content(&default_root_dir(), &name)
}

#[tauri::command]
pub fn skills_save_content(name: String, content: String) -> Result<SaveResult, String> {
  let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
  save_content(&default_root_dir(), &name, &content, &today)
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::time::{SystemTime, UNIX_EPOCH};

  use super::*;

  static COUNTER: AtomicUsize = AtomicUsize::new(0);

  fn temp_root() -> PathBuf {
    let mut root = std::env::temp_dir();
    let ts = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock")
      .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    root.push(format!("myskills-tauri-test-{ts}-{n}"));
    root
  }

  #[test]
  fn list_skills_reads_skill_metadata() {
    let root = temp_root();
    fs::create_dir_all(root.join("code-review")).expect("create skill dir");
    fs::write(
      root.join("code-review").join("SKILL.md"),
      r#"---
name: code-review
description: review code
category: quality
tags:
  - review
---

# Code Review
"#,
    )
    .expect("write skill");

    let skills = list_skills(&root).expect("list skills");
    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "code-review");
  }

  #[test]
  fn get_content_reads_frontmatter_and_body() {
    let root = temp_root();
    fs::create_dir_all(root.join("debug-helper")).expect("create skill dir");
    fs::write(
      root.join("debug-helper").join("SKILL.md"),
      r#"---
name: debug-helper
description: debug helper
---

## Steps
Do this.
"#,
    )
    .expect("write skill");

    let doc = get_content(&root, "debug-helper").expect("get content");
    assert_eq!(doc.frontmatter["name"], "debug-helper");
    assert!(doc.body.contains("## Steps"));
  }

  #[test]
  fn save_content_updates_last_updated() {
    let root = temp_root();
    fs::create_dir_all(root.join("planner")).expect("create skill dir");
    fs::write(
      root.join("planner").join("SKILL.md"),
      r#"---
name: planner
description: old
last_updated: "2026-01-01"
---

old body
"#,
    )
    .expect("write skill");

    let result = save_content(
      &root,
      "planner",
      r#"---
name: planner
description: new
---

new body
"#,
      "2026-02-27",
    )
    .expect("save content");

    assert!(result.success);
    let stored = fs::read_to_string(root.join("planner").join("SKILL.md")).expect("read saved");
    assert!(stored.contains("last_updated: 2026-02-27"));
    assert!(stored.contains("new body"));
  }
}
