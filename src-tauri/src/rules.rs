use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RulesContent {
  pub content: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RulesSaveResult {
  pub success: bool,
}

fn rules_file(root: &Path) -> PathBuf {
  root.join("AGENTS.md")
}

pub fn get_rules(root: &Path) -> Result<RulesContent, String> {
  let path = rules_file(root);
  if !path.exists() {
    return Ok(RulesContent {
      content: String::new(),
    });
  }

  let content = fs::read_to_string(path).map_err(|e| format!("Read AGENTS.md failed: {e}"))?;
  Ok(RulesContent { content })
}

pub fn save_rules(root: &Path, content: &str) -> Result<RulesSaveResult, String> {
  fs::create_dir_all(root).map_err(|e| format!("Create rules root failed: {e}"))?;
  let path = rules_file(root);
  fs::write(path, content).map_err(|e| format!("Write AGENTS.md failed: {e}"))?;
  Ok(RulesSaveResult { success: true })
}

#[tauri::command]
pub fn rules_get() -> Result<RulesContent, String> {
  get_rules(&crate::root_dir::default_root_dir())
}

#[tauri::command]
pub fn rules_save(content: String) -> Result<RulesSaveResult, String> {
  save_rules(&crate::root_dir::default_root_dir(), &content)
}

#[cfg(test)]
mod tests {
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
    root.push(format!("myskills-tauri-rules-test-{ts}-{n}"));
    root
  }

  #[test]
  fn get_rules_returns_empty_if_missing() {
    let root = temp_root();
    let content = get_rules(&root).expect("get rules");
    assert_eq!(content.content, "");
  }

  #[test]
  fn save_rules_writes_agents_file() {
    let root = temp_root();
    let next = "## Team Rules\nBe precise.\n";

    let result = save_rules(&root, next).expect("save rules");
    assert!(result.success);

    let stored = get_rules(&root).expect("read rules");
    assert_eq!(stored.content, next);
  }
}
