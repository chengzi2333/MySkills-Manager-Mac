use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct AppConfig {
  onboarding_completed: bool,
  skills_dir: String,
  auto_sync: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawAppConfig {
  onboarding_completed: Option<bool>,
  skills_dir: Option<String>,
  auto_sync: Option<bool>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingState {
  pub completed: bool,
  pub skills_dir: String,
  pub auto_sync: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct OnboardingSetSkillsDirResult {
  pub success: bool,
  pub skills: Vec<crate::skills::SkillMeta>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingCompleteResult {
  pub success: bool,
  pub auto_sync: bool,
  pub configured_tools: usize,
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

fn config_file(home: &Path) -> PathBuf {
  app_config_dir(home).join("config.json")
}

fn default_config(home: &Path) -> AppConfig {
  AppConfig {
    onboarding_completed: false,
    skills_dir: default_skills_root(home).to_string_lossy().to_string(),
    auto_sync: false,
  }
}

fn read_config(home: &Path) -> Result<AppConfig, String> {
  let path = config_file(home);
  if !path.exists() {
    return Ok(default_config(home));
  }

  let raw = fs::read_to_string(path).map_err(|e| format!("Read onboarding config failed: {e}"))?;
  if raw.trim().is_empty() {
    return Ok(default_config(home));
  }

  let parsed = serde_json::from_str::<RawAppConfig>(&raw)
    .map_err(|e| format!("Invalid onboarding config: {e}"))?;
  let defaults = default_config(home);
  Ok(AppConfig {
    onboarding_completed: parsed
      .onboarding_completed
      .unwrap_or(defaults.onboarding_completed),
    skills_dir: parsed.skills_dir.unwrap_or(defaults.skills_dir),
    auto_sync: parsed.auto_sync.unwrap_or(false),
  })
}

fn write_config(home: &Path, config: &AppConfig) -> Result<(), String> {
  fs::create_dir_all(app_config_dir(home))
    .map_err(|e| format!("Create app config dir failed: {e}"))?;
  let content = serde_json::to_string_pretty(config)
    .map_err(|e| format!("Serialize onboarding config failed: {e}"))?;
  fs::write(config_file(home), format!("{content}\n"))
    .map_err(|e| format!("Write onboarding config failed: {e}"))
}

pub fn apply_bootstrap_env() -> Result<(), String> {
  let home = default_home_dir();
  let config = read_config(&home)?;
  if !config.skills_dir.trim().is_empty() {
    unsafe {
      std::env::set_var("MYSKILLS_ROOT_DIR", config.skills_dir);
    }
  }
  Ok(())
}

pub fn onboarding_get_state_with_home(home: &Path) -> Result<OnboardingState, String> {
  let config = read_config(home)?;
  Ok(OnboardingState {
    completed: config.onboarding_completed,
    skills_dir: config.skills_dir,
    auto_sync: config.auto_sync,
  })
}

pub fn onboarding_set_skills_dir_with_home(
  home: &Path,
  dir: &str,
) -> Result<OnboardingSetSkillsDirResult, String> {
  let normalized = dir.trim();
  if normalized.is_empty() {
    return Err("skills dir is required".to_string());
  }

  let path = PathBuf::from(normalized);
  if !path.exists() {
    return Err("skills dir does not exist".to_string());
  }

  let mut config = read_config(home)?;
  config.skills_dir = path.to_string_lossy().to_string();
  config.onboarding_completed = false;
  write_config(home, &config)?;

  unsafe {
    std::env::set_var("MYSKILLS_ROOT_DIR", &config.skills_dir);
  }

  let skills = crate::skills::list_skills(&path)?;
  Ok(OnboardingSetSkillsDirResult {
    success: true,
    skills,
  })
}

pub fn onboarding_complete_with_home(
  home: &Path,
  auto_sync: bool,
) -> Result<OnboardingCompleteResult, String> {
  let mut config = read_config(home)?;
  let skills_dir = PathBuf::from(&config.skills_dir);
  if !skills_dir.exists() {
    return Err("skills dir does not exist".to_string());
  }

  config.onboarding_completed = true;
  config.auto_sync = auto_sync;
  write_config(home, &config)?;

  unsafe {
    std::env::set_var("MYSKILLS_ROOT_DIR", &config.skills_dir);
  }

  let mut configured_tools = 0usize;
  if auto_sync {
    let status = crate::setup::setup_status_with_home(home)?;
    let selected_tools = status
      .into_iter()
      .filter(|item| item.exists)
      .map(|item| item.id)
      .collect::<Vec<_>>();

    if !selected_tools.is_empty() {
      let results = crate::setup::apply_setup_with_paths(home, &skills_dir, &selected_tools, None)?;
      configured_tools = results.iter().filter(|item| item.success).count();
    }
  }

  Ok(OnboardingCompleteResult {
    success: true,
    auto_sync,
    configured_tools,
  })
}

#[tauri::command]
pub fn onboarding_get_state() -> Result<OnboardingState, String> {
  onboarding_get_state_with_home(&default_home_dir())
}

#[tauri::command]
pub fn onboarding_set_skills_dir(dir: String) -> Result<OnboardingSetSkillsDirResult, String> {
  onboarding_set_skills_dir_with_home(&default_home_dir(), &dir)
}

#[tauri::command]
pub fn onboarding_complete(auto_sync: bool) -> Result<OnboardingCompleteResult, String> {
  onboarding_complete_with_home(&default_home_dir(), auto_sync)
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
    root.push(format!("myskills-tauri-onboarding-test-{ts}-{n}"));
    root
  }

  #[test]
  fn onboarding_get_state_returns_default_when_missing() {
    let home = temp_home();
    let state = onboarding_get_state_with_home(&home).expect("get state");
    assert!(!state.completed);
    assert!(!state.skills_dir.is_empty());
    assert!(!state.auto_sync);
  }

  #[test]
  fn onboarding_set_skills_dir_returns_skill_list_and_persists() {
    let home = temp_home();
    let skills_root = home.join("my-skills");
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
      skills_root.join("code-review").join("SKILL.md"),
      "---\nname: code-review\n---\n",
    )
    .expect("write skill");

    let result = onboarding_set_skills_dir_with_home(&home, &skills_root.to_string_lossy())
      .expect("set skills dir");
    assert!(result.success);
    assert_eq!(result.skills.len(), 1);
    assert_eq!(result.skills[0].name, "code-review");

    let state = onboarding_get_state_with_home(&home).expect("get state");
    assert_eq!(PathBuf::from(state.skills_dir), skills_root);
    assert!(!state.completed);
  }

  #[test]
  fn onboarding_complete_manual_only_updates_state() {
    let home = temp_home();
    let skills_root = home.join("my-skills");
    fs::create_dir_all(skills_root.join("code-review")).expect("create skill dir");
    fs::write(
      skills_root.join("code-review").join("SKILL.md"),
      "---\nname: code-review\n---\n",
    )
    .expect("write skill");
    onboarding_set_skills_dir_with_home(&home, &skills_root.to_string_lossy())
      .expect("set dir");

    let result = onboarding_complete_with_home(&home, false).expect("complete");
    assert!(result.success);
    assert!(!result.auto_sync);
    assert_eq!(result.configured_tools, 0);

    let state = onboarding_get_state_with_home(&home).expect("get state");
    assert!(state.completed);
    assert!(!state.auto_sync);
  }
}
