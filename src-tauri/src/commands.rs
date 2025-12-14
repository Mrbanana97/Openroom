use std::path::{Path, PathBuf};

use tauri::async_runtime::spawn_blocking;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::image_io::{clear_preview_cache, load_or_create_thumbnail, render_preview_with_recipe};
use crate::metadata::read_metadata as read_exif_metadata;
use crate::models::{AssetSummary, EditRecipe, FolderIndex, GpuAdapter, Metadata};
use crate::recipe_io::{load_recipe_for_asset, save_recipe_for_asset};
use crate::state::{path_for, register_assets};

const SUPPORTED_EXTENSIONS: &[&str] = &[
    "dng", "nef", "cr2", "cr3", "arw", "raf", "rw2", "orf", "srw", "heic", "jpg", "jpeg", "png",
];

fn is_supported(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn to_asset_summary(path: PathBuf) -> Option<AssetSummary> {
    let file_name = path.file_name()?.to_string_lossy().to_string();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_uppercase();

    Some(AssetSummary {
        id: Uuid::new_v4().to_string(),
        file_name,
        extension,
        path: path.to_string_lossy().to_string(),
    })
}

fn collect_assets(folder: &Path) -> Result<Vec<AssetSummary>, String> {
    let mut assets: Vec<AssetSummary> = WalkDir::new(folder)
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_supported(entry.path()))
        .filter_map(|entry| to_asset_summary(entry.into_path()))
        .collect();

    assets.sort_by(|a, b| a.file_name.to_lowercase().cmp(&b.file_name.to_lowercase()));
    Ok(assets)
}

#[tauri::command]
pub async fn open_folder(path: String) -> Result<FolderIndex, String> {
    let res: Result<(PathBuf, Vec<AssetSummary>), String> = spawn_blocking(move || {
        let path_buf = PathBuf::from(&path);
        if !path_buf.is_dir() {
            return Err("Provided path is not a directory".into());
        }
        let assets = collect_assets(&path_buf)?;
        Ok((path_buf, assets))
    })
    .await
    .map_err(|e| e.to_string())?;

    let (path_buf, assets) = res?;

    clear_preview_cache();
    register_assets(
        assets
            .iter()
            .map(|asset| (asset.id.clone(), PathBuf::from(&asset.path))),
    );
    Ok(FolderIndex {
        id: Uuid::new_v4().to_string(),
        path: path_buf.to_string_lossy().to_string(),
        assets,
    })
}

#[tauri::command]
pub async fn get_thumbnail(asset_id: String) -> Result<Vec<u8>, String> {
    let path = path_for(&asset_id).ok_or("Asset not found")?;
    spawn_blocking(move || load_or_create_thumbnail(&asset_id, &path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn render_preview(
    asset_id: String,
    recipe: Option<EditRecipe>,
    max_dimension: Option<u32>,
) -> Result<Vec<u8>, String> {
    let path = path_for(&asset_id).ok_or("Asset not found")?;
    spawn_blocking(move || render_preview_with_recipe(&asset_id, &path, recipe, max_dimension))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn read_metadata(asset_id: String) -> Result<Metadata, String> {
    let path = path_for(&asset_id).ok_or("Asset not found")?;
    spawn_blocking(move || read_exif_metadata(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn save_recipe(asset_id: String, recipe: EditRecipe) -> Result<(), String> {
    let path = path_for(&asset_id).ok_or("Asset not found")?;
    spawn_blocking(move || save_recipe_for_asset(&path, &recipe))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn load_recipe(asset_id: String) -> Result<Option<EditRecipe>, String> {
    let path = path_for(&asset_id).ok_or("Asset not found")?;
    spawn_blocking(move || load_recipe_for_asset(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn detect_gpus() -> Result<Vec<GpuAdapter>, String> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });
    let adapters: Vec<GpuAdapter> = instance
        .enumerate_adapters(wgpu::Backends::all())
        .into_iter()
        .map(|adapter: wgpu::Adapter| {
            let info = adapter.get_info();
            GpuAdapter {
                name: info.name,
                backend: format!("{:?}", info.backend),
                device_type: format!("{:?}", info.device_type),
            }
        })
        .collect();
    Ok(adapters)
}
