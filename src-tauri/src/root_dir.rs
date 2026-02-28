use std::path::{Path, PathBuf};

pub fn default_root_dir() -> PathBuf {
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
