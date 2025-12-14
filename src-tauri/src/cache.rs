use std::fs;
use std::path::{Path, PathBuf};

use dirs::cache_dir;

pub fn cache_root() -> Result<PathBuf, String> {
    let base = cache_dir().ok_or("Unable to resolve cache directory")?;
    let root = base.join("openroom");
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

pub fn thumbnails_dir() -> Result<PathBuf, String> {
    let dir = cache_root()?.join("thumbs");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[allow(dead_code)]
pub fn previews_dir() -> Result<PathBuf, String> {
    let dir = cache_root()?.join("previews");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn cached_path(dir: &Path, asset_id: &str, suffix: &str) -> PathBuf {
    dir.join(format!("{asset_id}.{suffix}"))
}
