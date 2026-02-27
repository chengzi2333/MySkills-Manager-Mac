use chrono::{DateTime, Utc};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;
use std::path::PathBuf;

use crate::logs::LogEntry;
use crate::skills::list_skills;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct NamedCount {
  pub name: String,
  pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DayCount {
  pub date: String,
  pub count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StatsResult {
  pub total_invocations: usize,
  pub by_skill: Vec<NamedCount>,
  pub by_tool: Vec<NamedCount>,
  pub by_day: Vec<DayCount>,
  pub recent: Vec<LogEntry>,
  pub unused_skills: Vec<String>,
}

fn parse_ts(ts: &str) -> Option<DateTime<Utc>> {
  DateTime::parse_from_rfc3339(ts)
    .ok()
    .map(|dt| dt.with_timezone(&Utc))
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

fn compare_logs_desc(a: &LogEntry, b: &LogEntry) -> Ordering {
  match (parse_ts(&a.ts), parse_ts(&b.ts)) {
    (Some(a_ts), Some(b_ts)) => b_ts.cmp(&a_ts),
    _ => b.ts.cmp(&a.ts),
  }
}

pub fn compute_stats_with_now(
  root: &Path,
  days: u32,
  now: DateTime<Utc>,
) -> Result<StatsResult, String> {
  let logs = crate::logs::read_logs(root)?;
  let days = if days == 0 { 30 } else { days };
  let cutoff = now - chrono::Duration::days(days as i64);

  let mut filtered = logs
    .into_iter()
    .filter(|log| parse_ts(&log.ts).map(|ts| ts >= cutoff).unwrap_or(false))
    .collect::<Vec<_>>();

  filtered.sort_by(compare_logs_desc);

  let mut by_skill_map = HashMap::<String, usize>::new();
  let mut by_tool_map = HashMap::<String, usize>::new();
  let mut by_day_map = BTreeMap::<String, usize>::new();
  let mut used_skills = HashSet::<String>::new();

  for log in &filtered {
    *by_skill_map.entry(log.skill.clone()).or_insert(0) += 1;
    *by_tool_map.entry(log.tool.clone()).or_insert(0) += 1;
    used_skills.insert(log.skill.clone());

    if let Some(ts) = parse_ts(&log.ts) {
      let day = ts.format("%Y-%m-%d").to_string();
      *by_day_map.entry(day).or_insert(0) += 1;
    }
  }

  let mut by_skill = by_skill_map
    .into_iter()
    .map(|(name, count)| NamedCount { name, count })
    .collect::<Vec<_>>();
  by_skill.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));

  let mut by_tool = by_tool_map
    .into_iter()
    .map(|(name, count)| NamedCount { name, count })
    .collect::<Vec<_>>();
  by_tool.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));

  let by_day = by_day_map
    .into_iter()
    .map(|(date, count)| DayCount { date, count })
    .collect::<Vec<_>>();

  let recent = filtered.iter().take(10).cloned().collect::<Vec<_>>();

  let mut all_skills = list_skills(root)?
    .into_iter()
    .map(|skill| skill.name)
    .collect::<Vec<_>>();
  all_skills.sort();
  all_skills.dedup();

  let unused_skills = all_skills
    .into_iter()
    .filter(|name| !used_skills.contains(name))
    .collect::<Vec<_>>();

  Ok(StatsResult {
    total_invocations: filtered.len(),
    by_skill,
    by_tool,
    by_day,
    recent,
    unused_skills,
  })
}

#[tauri::command]
pub fn stats_get(days: Option<u32>) -> Result<StatsResult, String> {
  compute_stats_with_now(&default_root_dir(), days.unwrap_or(30), Utc::now())
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use std::time::{SystemTime, UNIX_EPOCH};

  use chrono::TimeZone;

  use super::*;

  static COUNTER: AtomicUsize = AtomicUsize::new(0);

  fn temp_root() -> PathBuf {
    let mut root = std::env::temp_dir();
    let ts = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock")
      .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    root.push(format!("myskills-tauri-stats-test-{ts}-{n}"));
    root
  }

  fn seed(root: &Path) {
    fs::create_dir_all(root.join(".logs")).expect("create logs dir");
    fs::write(
      root.join(".logs").join("skill-usage.jsonl"),
      r#"{"ts":"2026-02-25T01:00:00Z","skill":"code-review","cwd":"/tmp/a","tool":"codex"}
{"ts":"2026-02-26T02:00:00Z","skill":"debug-helper","cwd":"/tmp/b","tool":"claude-code"}
{"ts":"2026-02-27T03:00:00Z","skill":"code-review","cwd":"/tmp/c","tool":"codex"}
"#,
    )
    .expect("write logs");

    fs::create_dir_all(root.join("code-review")).expect("create skill");
    fs::write(
      root.join("code-review").join("SKILL.md"),
      r#"---
name: code-review
description: review
---
"#,
    )
    .expect("write skill");
    fs::create_dir_all(root.join("debug-helper")).expect("create skill");
    fs::write(
      root.join("debug-helper").join("SKILL.md"),
      r#"---
name: debug-helper
description: debug
---
"#,
    )
    .expect("write skill");
    fs::create_dir_all(root.join("planner")).expect("create skill");
    fs::write(
      root.join("planner").join("SKILL.md"),
      r#"---
name: planner
description: plan
---
"#,
    )
    .expect("write skill");
  }

  #[test]
  fn compute_stats_aggregates_expected_fields() {
    let root = temp_root();
    seed(&root);
    let now = Utc.with_ymd_and_hms(2026, 2, 27, 12, 0, 0).unwrap();

    let stats = compute_stats_with_now(&root, 30, now).expect("compute stats");

    assert_eq!(stats.total_invocations, 3);
    assert_eq!(stats.by_skill[0], NamedCount {
      name: "code-review".to_string(),
      count: 2
    });
    assert_eq!(stats.by_tool[0].name, "codex");
    assert_eq!(stats.by_tool[0].count, 2);
    assert_eq!(stats.by_day.len(), 3);
    assert_eq!(stats.recent.len(), 3);
    assert_eq!(stats.recent[0].ts, "2026-02-27T03:00:00Z");
    assert_eq!(stats.unused_skills, vec!["planner".to_string()]);
  }

  #[test]
  fn compute_stats_applies_days_window() {
    let root = temp_root();
    seed(&root);
    let now = Utc.with_ymd_and_hms(2026, 2, 27, 12, 0, 0).unwrap();

    let stats = compute_stats_with_now(&root, 1, now).expect("compute stats");

    assert_eq!(stats.total_invocations, 1);
    assert_eq!(stats.by_day.len(), 1);
    assert_eq!(stats.by_day[0].date, "2026-02-27");
  }

  #[test]
  fn parse_ts_supports_rfc3339() {
    let parsed = parse_ts("2026-02-27T03:00:00Z").expect("parse timestamp");
    assert_eq!(parsed.to_rfc3339(), "2026-02-27T03:00:00+00:00");
  }
}
