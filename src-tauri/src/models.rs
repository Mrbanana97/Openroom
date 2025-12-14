use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSummary {
    pub id: String,
    pub file_name: String,
    pub extension: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderIndex {
    pub id: String,
    pub path: String,
    pub assets: Vec<AssetSummary>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub camera: Option<String>,
    pub lens: Option<String>,
    pub iso: Option<String>,
    pub shutter: Option<String>,
    pub aperture: Option<String>,
    pub focal: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GlobalAdjustments {
    pub exposure_ev: f32,
    pub contrast: f32,
    pub highlights: f32,
    pub shadows: f32,
    pub whites: f32,
    pub blacks: f32,
    pub temp: f32,
    pub tint: f32,
    pub vibrance: f32,
    pub saturation: f32,
}

impl Default for GlobalAdjustments {
    fn default() -> Self {
        Self {
            exposure_ev: 0.0,
            contrast: 0.0,
            highlights: 0.0,
            shadows: 0.0,
            whites: 0.0,
            blacks: 0.0,
            temp: 0.0,
            tint: 0.0,
            vibrance: 0.0,
            saturation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalAdjustments {
    pub exposure_ev: f32,
    pub temp: f32,
    pub tint: f32,
    pub saturation: f32,
}

impl Default for LocalAdjustments {
    fn default() -> Self {
        Self {
            exposure_ev: 0.0,
            temp: 0.0,
            tint: 0.0,
            saturation: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Mask {
    pub mask_type: String, // "linear_gradient"
    pub start: (f32, f32), // normalized 0..1
    pub end: (f32, f32),
    pub feather: f32, // 0..1
    pub invert: bool,
}

impl Default for Mask {
    fn default() -> Self {
        Self {
            mask_type: "linear_gradient".into(),
            start: (0.3, 0.2),
            end: (0.7, 0.8),
            feather: 0.2,
            invert: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AdjustmentLayer {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub opacity: f32,
    pub mask: Mask,
    pub adjustments: LocalAdjustments,
}

impl Default for AdjustmentLayer {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Gradient".into(),
            enabled: true,
            opacity: 1.0,
            mask: Mask::default(),
            adjustments: LocalAdjustments::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EditRecipe {
    pub version: u8,
    pub globals: GlobalAdjustments,
    pub layers: Vec<AdjustmentLayer>,
}

impl Default for EditRecipe {
    fn default() -> Self {
        Self {
            version: 1,
            globals: GlobalAdjustments::default(),
            layers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuAdapter {
    pub name: String,
    pub backend: String,
    pub device_type: String,
}
