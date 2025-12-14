use std::path::PathBuf;

use dashmap::DashMap;
use once_cell::sync::Lazy;

pub static ASSET_REGISTRY: Lazy<DashMap<String, PathBuf>> = Lazy::new(DashMap::new);

pub fn register_assets<I>(assets: I)
where
    I: IntoIterator<Item = (String, PathBuf)>,
{
    ASSET_REGISTRY.clear();
    for (id, path) in assets {
        ASSET_REGISTRY.insert(id, path);
    }
}

pub fn path_for(id: &str) -> Option<PathBuf> {
    ASSET_REGISTRY.get(id).map(|entry| entry.value().clone())
}
