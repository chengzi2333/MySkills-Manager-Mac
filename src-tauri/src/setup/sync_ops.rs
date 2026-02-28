use std::fs;
use std::path::Path;

pub(super) fn remove_if_exists(path: &Path) -> Result<(), String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(_) => return Ok(()),
    };

    if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir_all(path).map_err(|e| format!("Remove dir failed: {e}"))?;
    } else {
        fs::remove_file(path).map_err(|e| format!("Remove file failed: {e}"))?;
    }
    Ok(())
}

pub(super) fn backup_if_exists(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let file_name = path
        .file_name()
        .ok_or_else(|| "Backup target must include file name".to_string())?
        .to_string_lossy()
        .to_string();
    let backup_path = path.with_file_name(format!("{file_name}.bak"));
    fs::copy(path, &backup_path).map_err(|e| format!("Create backup failed: {e}"))?;
    Ok(())
}

pub(super) fn register_rollback_path(paths: &mut Vec<super::RollbackPath>, path: &Path) {
    if paths.iter().any(|entry| entry.path == path) {
        return;
    }
    paths.push(super::RollbackPath {
        path: path.to_path_buf(),
        existed: path.exists(),
    });
}

fn rollback_config_paths(paths: &[super::RollbackPath]) -> Result<(), String> {
    let mut errors = Vec::<String>::new();
    for entry in paths.iter().rev() {
        let file_name = match entry.path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => {
                errors.push(format!(
                    "Rollback target must include file name: {}",
                    entry.path.to_string_lossy()
                ));
                continue;
            }
        };
        let backup_path = entry.path.with_file_name(format!("{file_name}.bak"));
        if backup_path.exists() {
            if let Err(err) = fs::copy(&backup_path, &entry.path) {
                errors.push(format!(
                    "Restore backup failed ({}): {err}",
                    entry.path.to_string_lossy()
                ));
            }
            continue;
        }

        if !entry.existed && entry.path.exists() {
            if let Err(err) = remove_if_exists(&entry.path) {
                errors.push(format!(
                    "Remove created file failed ({}): {err}",
                    entry.path.to_string_lossy()
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

pub(super) fn finalize_with_rollback(
    mut out: Vec<super::ApplyResult>,
    mut failure: super::ApplyResult,
    rollback_paths: &[super::RollbackPath],
) -> Vec<super::ApplyResult> {
    if let Err(rollback_err) = rollback_config_paths(rollback_paths) {
        failure.error = Some(match failure.error.take() {
            Some(existing) => format!("{existing}; rollback failed: {rollback_err}"),
            None => format!("rollback failed: {rollback_err}"),
        });
    }
    out.push(failure);
    out
}

#[cfg(target_family = "windows")]
fn create_symlink_file(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::windows::fs::symlink_file(source, target)
}

#[cfg(target_family = "unix")]
fn create_symlink_file(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

pub(super) fn sync_skill_file(source: &Path, target: &Path) -> Result<String, String> {
    remove_if_exists(target)?;
    match create_symlink_file(source, target) {
        Ok(_) => Ok("symlink".to_string()),
        Err(_) => {
            fs::copy(source, target).map_err(|e| format!("Copy skill file failed: {e}"))?;
            Ok("copy".to_string())
        }
    }
}

pub(super) fn remove_skill_target(target_dir: &Path, target_file: &Path) -> Result<(), String> {
    remove_if_exists(target_file)?;
    if target_dir.exists() {
        let mut entries =
            fs::read_dir(target_dir).map_err(|e| format!("Read target dir failed: {e}"))?;
        if entries.next().is_none() {
            fs::remove_dir_all(target_dir).map_err(|e| format!("Remove target dir failed: {e}"))?;
        }
    }
    Ok(())
}
