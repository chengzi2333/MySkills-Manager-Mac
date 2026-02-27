use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogEntry {
  pub ts: String,
  pub skill: String,
  pub cwd: String,
  pub tool: String,
  pub session: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LogsResult {
  pub logs: Vec<LogEntry>,
  pub total: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LogsQuery {
  pub skill: Option<String>,
  pub tool: Option<String>,
  pub from: Option<String>,
  pub to: Option<String>,
  pub page: usize,
  pub limit: usize,
}

fn parse_ts(ts: &str) -> Option<DateTime<Utc>> {
  DateTime::parse_from_rfc3339(ts)
    .ok()
    .map(|dt| dt.with_timezone(&Utc))
}

fn logs_file(root: &Path) -> PathBuf {
  root.join(".logs").join("skill-usage.jsonl")
}

pub(crate) fn read_logs(root: &Path) -> Result<Vec<LogEntry>, String> {
  let file_path = logs_file(root);
  if !file_path.exists() {
    return Ok(Vec::new());
  }

  let file = File::open(&file_path).map_err(|e| format!("Open log file failed: {e}"))?;
  let reader = BufReader::new(file);
  let mut logs = Vec::new();

  for line in reader.lines() {
    let line = line.map_err(|e| format!("Read log line failed: {e}"))?;
    if line.trim().is_empty() {
      continue;
    }
    if let Ok(log) = serde_json::from_str::<LogEntry>(&line) {
      logs.push(log);
    }
  }

  Ok(logs)
}

fn compare_ts_desc(a: &LogEntry, b: &LogEntry) -> Ordering {
  match (parse_ts(&a.ts), parse_ts(&b.ts)) {
    (Some(a_ts), Some(b_ts)) => b_ts.cmp(&a_ts),
    _ => b.ts.cmp(&a.ts),
  }
}

pub fn get_logs(root: &Path, query: &LogsQuery) -> Result<LogsResult, String> {
  let mut logs = read_logs(root)?;
  let from = query.from.as_deref().and_then(parse_ts);
  let to = query.to.as_deref().and_then(parse_ts);

  logs.retain(|log| {
    if let Some(skill) = query.skill.as_deref() {
      if log.skill != skill {
        return false;
      }
    }

    if let Some(tool) = query.tool.as_deref() {
      if log.tool != tool {
        return false;
      }
    }

    if from.is_none() && to.is_none() {
      return true;
    }

    let Some(ts) = parse_ts(&log.ts) else {
      return false;
    };

    if let Some(from_ts) = from {
      if ts < from_ts {
        return false;
      }
    }
    if let Some(to_ts) = to {
      if ts > to_ts {
        return false;
      }
    }

    true
  });

  logs.sort_by(compare_ts_desc);

  let total = logs.len();
  let page = if query.page == 0 { 1 } else { query.page };
  let limit = if query.limit == 0 {
    100
  } else {
    query.limit.min(1000)
  };
  let start = (page - 1).saturating_mul(limit);
  let rows = logs.into_iter().skip(start).take(limit).collect::<Vec<_>>();

  Ok(LogsResult { logs: rows, total })
}

#[tauri::command]
pub fn logs_get(
  skill: Option<String>,
  tool: Option<String>,
  from: Option<String>,
  to: Option<String>,
  page: Option<usize>,
  limit: Option<usize>,
) -> Result<LogsResult, String> {
  let query = LogsQuery {
    skill,
    tool,
    from,
    to,
    page: page.unwrap_or(1),
    limit: limit.unwrap_or(100),
  };
  get_logs(&crate::root_dir::default_root_dir(), &query)
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
    root.push(format!("myskills-tauri-logs-test-{ts}-{n}"));
    root
  }

  fn seed_logs(root: &Path) {
    let logs_dir = root.join(".logs");
    fs::create_dir_all(&logs_dir).expect("create logs dir");
    fs::write(
      logs_dir.join("skill-usage.jsonl"),
      r#"{"ts":"2026-02-26T01:00:00Z","skill":"code-review","cwd":"/tmp/a","tool":"codex"}
{"ts":"2026-02-26T02:00:00Z","skill":"debug-helper","cwd":"/tmp/b","tool":"claude-code"}
{"ts":"2026-02-27T03:00:00Z","skill":"code-review","cwd":"/tmp/c","tool":"codex"}
"#,
    )
    .expect("write logs");
  }

  #[test]
  fn get_logs_filters_by_skill_and_tool() {
    let root = temp_root();
    seed_logs(&root);

    let result = get_logs(
      &root,
      &LogsQuery {
        skill: Some("code-review".to_string()),
        tool: Some("codex".to_string()),
        from: None,
        to: None,
        page: 1,
        limit: 50,
      },
    )
    .expect("query logs");

    assert_eq!(result.total, 2);
    assert_eq!(result.logs.len(), 2);
    assert_eq!(result.logs[0].skill, "code-review");
    assert_eq!(result.logs[0].tool, "codex");
  }

  #[test]
  fn get_logs_applies_time_range_and_pagination() {
    let root = temp_root();
    seed_logs(&root);

    let result = get_logs(
      &root,
      &LogsQuery {
        skill: None,
        tool: None,
        from: Some("2026-02-26T00:30:00Z".to_string()),
        to: Some("2026-02-27T00:00:00Z".to_string()),
        page: 1,
        limit: 1,
      },
    )
    .expect("query logs");

    assert_eq!(result.total, 2);
    assert_eq!(result.logs.len(), 1);
    assert_eq!(result.logs[0].ts, "2026-02-26T02:00:00Z");
  }
}
