use std::path::{Path, PathBuf};

pub fn default_home_dir() -> PathBuf {
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("USERPROFILE") {
        return PathBuf::from(home);
    }
    PathBuf::from("./")
}

pub fn default_skills_root(home: &Path) -> PathBuf {
    if let Ok(path) = std::env::var("MYSKILLS_ROOT_DIR") {
        return PathBuf::from(path);
    }
    home.join("my-skills")
}

pub fn app_config_dir(home: &Path) -> PathBuf {
    home.join(".myskills-manager")
}

pub fn default_root_dir() -> PathBuf {
    default_skills_root(&default_home_dir())
}
