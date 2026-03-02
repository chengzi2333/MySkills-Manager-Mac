use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::{Ordering, Reverse};
use std::collections::BinaryHeap;
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

fn parse_log_line(line: &str) -> Option<LogEntry> {
    if let Ok(log) = serde_json::from_str::<LogEntry>(line) {
        return Some(log);
    }

    if line.contains('\\') {
        let escaped = line.replace('\\', "\\\\");
        if let Ok(log) = serde_json::from_str::<LogEntry>(&escaped) {
            return Some(log);
        }
    }

    None
}

pub(crate) fn for_each_log(
    root: &Path,
    mut handler: impl FnMut(LogEntry),
) -> Result<(), String> {
    let file_path = logs_file(root);
    if !file_path.exists() {
        return Ok(());
    }

    let file = File::open(&file_path).map_err(|e| format!("Open log file failed: {e}"))?;
    let reader = BufReader::new(file);
    let mut reader = reader;
    let mut raw_line = Vec::<u8>::new();
    loop {
        raw_line.clear();
        let read = reader
            .read_until(b'\n', &mut raw_line)
            .map_err(|e| format!("Read log line failed: {e}"))?;
        if read == 0 {
            break;
        }
        while matches!(raw_line.last(), Some(b'\n' | b'\r')) {
            raw_line.pop();
        }
        if raw_line.is_empty() {
            continue;
        }

        let line = String::from_utf8_lossy(&raw_line);
        if let Some(log) = parse_log_line(line.as_ref()) {
            handler(log);
        }
    }

    Ok(())
}

fn compare_ts_desc(a: &LogEntry, b: &LogEntry) -> Ordering {
    match (parse_ts(&a.ts), parse_ts(&b.ts)) {
        (Some(a_ts), Some(b_ts)) => b_ts.cmp(&a_ts),
        _ => b.ts.cmp(&a.ts),
    }
}

fn compare_ts_asc(a: &LogEntry, b: &LogEntry) -> Ordering {
    compare_ts_desc(b, a)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RankedLog(LogEntry);

impl Ord for RankedLog {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_ts_asc(&self.0, &other.0)
    }
}

impl PartialOrd for RankedLog {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn get_logs(root: &Path, query: &LogsQuery) -> Result<LogsResult, String> {
    if let Ok(indexed) = crate::log_index::query_logs_index(root, query) {
        return Ok(indexed);
    }

    let from = query.from.as_deref().and_then(parse_ts);
    let to = query.to.as_deref().and_then(parse_ts);
    let page = if query.page == 0 { 1 } else { query.page };
    let limit = if query.limit == 0 {
        100
    } else {
        query.limit.min(1000)
    };
    let start = (page - 1).saturating_mul(limit);
    let window = start.saturating_add(limit);
    let mut total = 0usize;
    let mut heap = BinaryHeap::<Reverse<RankedLog>>::new();

    for_each_log(root, |log| {
        if let Some(skill) = query.skill.as_deref() {
            if log.skill != skill {
                return;
            }
        }

        if let Some(tool) = query.tool.as_deref() {
            if log.tool != tool {
                return;
            }
        }

        if from.is_some() || to.is_some() {
            let Some(ts) = parse_ts(&log.ts) else {
                return;
            };

            if let Some(from_ts) = from.as_ref() {
                if ts < *from_ts {
                    return;
                }
            }
            if let Some(to_ts) = to.as_ref() {
                if ts > *to_ts {
                    return;
                }
            }
        }

        total += 1;
        if window == 0 {
            return;
        }
        if heap.len() < window {
            heap.push(Reverse(RankedLog(log)));
            return;
        }

        if let Some(oldest) = heap.peek() {
            if compare_ts_desc(&log, &oldest.0.0) == Ordering::Less {
                let _ = heap.pop();
                heap.push(Reverse(RankedLog(log)));
            }
        }
    })?;

    let mut top_window = heap
        .into_iter()
        .map(|entry| entry.0.0)
        .collect::<Vec<_>>();
    top_window.sort_by(compare_ts_desc);
    let rows = top_window
        .into_iter()
        .skip(start)
        .take(limit)
        .collect::<Vec<_>>();

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

    #[test]
    fn read_logs_recovers_windows_style_unescaped_paths() {
        let root = temp_root();
        let logs_dir = root.join(".logs");
        fs::create_dir_all(&logs_dir).expect("create logs dir");
        fs::write(
            logs_dir.join("skill-usage.jsonl"),
            r#"{"ts":"2026-02-27T10:00:00Z","skill":"brainstorming","cwd":"C:\Own Docm\Coding\My-Skills","tool":"codex"}"#,
        )
        .expect("write logs");

        let mut result = Vec::<LogEntry>::new();
        for_each_log(&root, |log| result.push(log)).expect("read logs");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].skill, "brainstorming");
        assert_eq!(result[0].tool, "codex");
        assert_eq!(result[0].cwd, r"C:\Own Docm\Coding\My-Skills");
    }

    #[test]
    fn for_each_log_tolerates_non_utf8_lines() {
        let root = temp_root();
        let logs_dir = root.join(".logs");
        fs::create_dir_all(&logs_dir).expect("create logs dir");

        let mut raw = Vec::<u8>::new();
        raw.extend_from_slice(
            br#"{"ts":"2026-03-02T04:08:09Z","skill":"using-superpowers","cwd":"C:\Own Docm\Coding\My-Skills","tool":"codex"}"#,
        );
        raw.push(b'\n');
        raw.extend_from_slice(
            br#"{"ts":"2026-03-02T04:08:09Z","skill":"writing-plans","cwd":"C:\Own Docm\Coding\bad"#,
        );
        raw.push(0xB7);
        raw.extend_from_slice(br#"path","tool":"codex"}"#);
        raw.push(b'\n');

        fs::write(logs_dir.join("skill-usage.jsonl"), raw).expect("write mixed-encoding logs");

        let mut result = Vec::<LogEntry>::new();
        for_each_log(&root, |log| result.push(log)).expect("read mixed-encoding logs");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].skill, "using-superpowers");
        assert_eq!(result[1].skill, "writing-plans");
    }

    #[test]
    fn get_logs_must_not_load_full_logs_vector_before_filtering() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let source =
            fs::read_to_string(manifest_dir.join("src").join("logs.rs")).expect("read logs.rs");
        let main = source
            .split("#[cfg(test)]")
            .next()
            .expect("logs main section");

        assert!(
            !main.contains("let mut logs = read_logs(root)?;"),
            "get_logs should not read all logs into a full vector before filtering"
        );
        assert!(
            main.contains("crate::log_index::query_logs_index(root, query)"),
            "get_logs should try indexed query path before fallback scanning"
        );
    }
}
