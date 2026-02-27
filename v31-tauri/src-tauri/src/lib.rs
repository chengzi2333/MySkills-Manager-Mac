mod skills;
mod logs;
mod stats;
mod rules;
mod git;
mod setup;

#[tauri::command]
fn app_ping() -> String {
  "pong".to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      app_ping,
      skills::skills_list,
      skills::skills_get_content,
      skills::skills_save_content,
      logs::logs_get,
      stats::stats_get,
      rules::rules_get,
      rules::rules_save,
      git::git_status,
      git::git_commit,
      git::git_push,
      setup::setup_status,
      setup::setup_apply,
      setup::setup_get_custom_tools,
      setup::setup_add_custom_tool,
      setup::setup_remove_custom_tool
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
  #[test]
  fn app_ping_returns_pong() {
    assert_eq!(super::app_ping(), "pong");
  }
}
