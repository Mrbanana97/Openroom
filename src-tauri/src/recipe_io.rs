use std::fs;
use std::path::{Path, PathBuf};

use crate::models::EditRecipe;

fn sidecar_path(asset_path: &Path) -> PathBuf {
    let mut file_name = asset_path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "edit".to_string());
    file_name.push_str(".lumen.json");
    asset_path
        .parent()
        .map(|p| p.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}

pub fn save_recipe_for_asset(asset_path: &Path, recipe: &EditRecipe) -> Result<(), String> {
    let path = sidecar_path(asset_path);
    let serialized = serde_json::to_string_pretty(recipe)
        .map_err(|e| format!("Serialize recipe failed: {e}"))?;
    fs::write(&path, serialized).map_err(|e| format!("Write sidecar failed: {e}"))
}

pub fn load_recipe_for_asset(asset_path: &Path) -> Result<Option<EditRecipe>, String> {
    let path = sidecar_path(asset_path);
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Read sidecar failed: {e}"))?;
    let recipe: EditRecipe =
        serde_json::from_str(&data).map_err(|e| format!("Parse sidecar failed: {e}"))?;
    Ok(Some(recipe))
}
