use std::collections::VecDeque;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};

use dashmap::DashMap;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::imageops::{self, FilterType as ResizeFilter};
use image::{ColorType, DynamicImage, ImageEncoder, Rgba, RgbaImage};
use libraw::{ProcessedImage, Processor};
use once_cell::sync::Lazy;
use rawloader::decode_file as decode_raw_file;
use rawloader::{decode_dummy, RawImage, RawImageData};
use rayon::prelude::*;

use crate::cache::{cached_path, thumbnails_dir};
use crate::gpu;
use crate::models::{AdjustmentLayer, EditRecipe, GlobalAdjustments};

// cache decoded previews to avoid re-decoding per slider move
type PreviewBuf = Arc<RgbaImage>;
#[derive(Clone)]
struct CachedPreview {
    buf: PreviewBuf,
    max_dim: u32,
}
static PREVIEW_MASTERS: Lazy<DashMap<String, CachedPreview>> = Lazy::new(DashMap::new);
static PREVIEW_VARIANTS: Lazy<DashMap<String, PreviewBuf>> = Lazy::new(DashMap::new);
static PREVIEW_LRU: Lazy<Mutex<VecDeque<String>>> = Lazy::new(|| Mutex::new(VecDeque::new()));
const PREVIEW_CACHE_ASSETS: usize = 2;
const PREVIEW_MIN_DIM: u32 = 480;
const PREVIEW_MAX_DIM: u32 = 3200;
const PREVIEW_MASTER_BASE: u32 = 1920;

fn cache_key(asset_id: &str, max_dimension: u32) -> String {
    format!("{asset_id}:{max_dimension}")
}

fn normalize_dimension(dim: u32) -> u32 {
    dim.clamp(PREVIEW_MIN_DIM, PREVIEW_MAX_DIM)
}

fn target_size(w: u32, h: u32, max_dimension: u32) -> (u32, u32) {
    if w == 0 || h == 0 {
        return (1, 1);
    }
    if w >= h {
        let nh = ((max_dimension as f32 / w as f32) * h as f32) as u32;
        (max_dimension, nh.max(1))
    } else {
        let nw = ((max_dimension as f32 / h as f32) * w as f32) as u32;
        (nw.max(1), max_dimension)
    }
}

fn touch_asset(asset_id: &str) {
    if let Ok(mut lru) = PREVIEW_LRU.lock() {
        if let Some(pos) = lru.iter().position(|id| id == asset_id) {
            lru.remove(pos);
        }
        lru.push_back(asset_id.to_string());
    }
}

fn evict_if_needed() {
    let mut evicted: Vec<String> = Vec::new();
    if let Ok(mut lru) = PREVIEW_LRU.lock() {
        while lru.len() > PREVIEW_CACHE_ASSETS {
            if let Some(id) = lru.pop_front() {
                evicted.push(id);
            } else {
                break;
            }
        }
    }
    for id in evicted {
        PREVIEW_MASTERS.remove(&id);
        let prefix = format!("{id}:");
        PREVIEW_VARIANTS.retain(|k, _| !k.starts_with(&prefix));
    }
}

fn drop_variants_for(asset_id: &str) {
    let prefix = format!("{asset_id}:");
    PREVIEW_VARIANTS.retain(|k, _| !k.starts_with(&prefix));
}

fn resize_rgba_preserve_aspect(img: &RgbaImage, max_dimension: u32) -> RgbaImage {
    let max_dimension = max_dimension.max(1);
    let (nw, nh) = target_size(img.width(), img.height(), max_dimension);
    if nw == img.width() && nh == img.height() {
        return img.clone();
    }

    if gpu::available() {
        if let Some(out) = gpu::resize_rgba(img, nw, nh) {
            return out;
        }
    }

    imageops::resize(img, nw, nh, ResizeFilter::CatmullRom)
}

fn store_master(asset_id: &str, img: RgbaImage) -> CachedPreview {
    let max_dim = img.width().max(img.height()).max(1);
    let entry = CachedPreview {
        buf: Arc::new(img),
        max_dim,
    };
    PREVIEW_MASTERS.insert(asset_id.to_string(), entry.clone());
    drop_variants_for(asset_id);
    touch_asset(asset_id);
    evict_if_needed();
    entry
}

fn master_preview(
    asset_id: &str,
    path: &Path,
    requested_dim: u32,
) -> Result<CachedPreview, String> {
    let target = normalize_dimension(requested_dim);
    if let Some(hit) = PREVIEW_MASTERS.get(asset_id) {
        if target <= hit.max_dim {
            touch_asset(asset_id);
            return Ok(hit.clone());
        }
    }

    let decode_target = target.max(PREVIEW_MASTER_BASE).min(PREVIEW_MAX_DIM);
    let decoded = render_resized(path, decode_target)?;
    Ok(store_master(asset_id, decoded))
}

fn scaled_preview(asset_id: &str, path: &Path, requested_dim: u32) -> Result<PreviewBuf, String> {
    let target = normalize_dimension(requested_dim);
    let master = master_preview(asset_id, path, target)?;
    let master_dim = master.max_dim;

    if target >= master_dim.saturating_sub(4) {
        return Ok(master.buf);
    }

    let key = cache_key(asset_id, target);
    if let Some(existing) = PREVIEW_VARIANTS.get(&key) {
        touch_asset(asset_id);
        return Ok(existing.clone());
    }

    let resized = resize_rgba_preserve_aspect(&master.buf, target);
    let arc = Arc::new(resized);
    PREVIEW_VARIANTS.insert(key, arc.clone());
    touch_asset(asset_id);
    evict_if_needed();
    Ok(arc)
}

fn channels_from_len(len: usize, w: u32, h: u32) -> Option<usize> {
    let pixels = (w as usize).saturating_mul(h as usize);
    if pixels == 0 {
        return None;
    }
    let (div, rem) = (len / pixels, len % pixels);
    if div == 0 || rem != 0 {
        None
    } else {
        Some(div)
    }
}

fn libraw_to_rgba_u8(img: &ProcessedImage<u8>) -> Result<RgbaImage, String> {
    let w = img.width();
    let h = img.height();
    let data: &[u8] = img;
    let channels = channels_from_len(data.len(), w, h).ok_or_else(|| {
        format!(
            "LibRaw returned unexpected buffer size ({} bytes for {}x{})",
            data.len(),
            w,
            h
        )
    })?;

    let mut rgba = RgbaImage::new(w, h);
    for (idx, pixel) in rgba.pixels_mut().enumerate() {
        let base = idx * channels;
        let (r, g, b, a) = match channels {
            1 => {
                let v = *data.get(base).unwrap_or(&0);
                (v, v, v, 255)
            }
            2 => {
                let v = *data.get(base).unwrap_or(&0);
                let a = *data.get(base + 1).unwrap_or(&255);
                (v, v, v, a)
            }
            _ => {
                let r = *data.get(base).unwrap_or(&0);
                let g = *data.get(base + 1).unwrap_or(&r);
                let b = *data.get(base + 2).unwrap_or(&g);
                let a = if channels > 3 {
                    *data.get(base + 3).unwrap_or(&255)
                } else {
                    255
                };
                (r, g, b, a)
            }
        };
        *pixel = Rgba([r, g, b, a]);
    }
    Ok(rgba)
}

fn libraw_to_rgba_u16(img: &ProcessedImage<u16>) -> Result<RgbaImage, String> {
    let w = img.width();
    let h = img.height();
    let data: &[u16] = img;
    let channels = channels_from_len(data.len(), w, h).ok_or_else(|| {
        format!(
            "LibRaw returned unexpected buffer size ({} samples for {}x{})",
            data.len(),
            w,
            h
        )
    })?;

    let mut rgba = RgbaImage::new(w, h);
    let to_byte = |v: u16| -> u8 { (v >> 8) as u8 };

    for (idx, pixel) in rgba.pixels_mut().enumerate() {
        let base = idx * channels;
        let (r16, g16, b16, a16) = match channels {
            1 => {
                let v = *data.get(base).unwrap_or(&0);
                (v, v, v, 65535)
            }
            2 => {
                let v = *data.get(base).unwrap_or(&0);
                let a = *data.get(base + 1).unwrap_or(&65535);
                (v, v, v, a)
            }
            _ => {
                let r = *data.get(base).unwrap_or(&0);
                let g = *data.get(base + 1).unwrap_or(&r);
                let b = *data.get(base + 2).unwrap_or(&g);
                let a = if channels > 3 {
                    *data.get(base + 3).unwrap_or(&65535)
                } else {
                    65535
                };
                (r, g, b, a)
            }
        };
        *pixel = Rgba([to_byte(r16), to_byte(g16), to_byte(b16), to_byte(a16)]);
    }
    Ok(rgba)
}

fn decode_with_libraw(bytes: &[u8]) -> Result<DynamicImage, String> {
    match Processor::new().process_16bit(bytes) {
        Ok(processed) => {
            let rgba = libraw_to_rgba_u16(&processed)?;
            Ok(DynamicImage::ImageRgba8(rgba))
        }
        Err(err16) => match Processor::new().process_8bit(bytes) {
            Ok(processed) => {
                let rgba = libraw_to_rgba_u8(&processed)?;
                Ok(DynamicImage::ImageRgba8(rgba))
            }
            Err(err8) => Err(format!(
                "LibRaw decode failed (16-bit: {err16}; 8-bit: {err8})"
            )),
        },
    }
}

fn load_dynamic_image(path: &Path) -> Result<DynamicImage, String> {
    match image::open(path) {
        Ok(img) => Ok(img),
        Err(primary) => {
            // Fallback 1: try loading from raw bytes to handle uppercase/ext edge cases
            let bytes = fs::read(path).map_err(|e| format!("Failed to read image bytes: {e}"))?;
            if let Ok(img_mem) = image::load_from_memory(&bytes) {
                return Ok(img_mem);
            }

            // Fallback 2: LibRaw for broad RAW coverage (ARW/DNG/CR3...)
            let libraw_err = match decode_with_libraw(&bytes) {
                Ok(img) => return Ok(img),
                Err(err) => err,
            };

            // Fallback 3: rawloader for RAW formats
            match decode_raw_file(path) {
                Ok(raw) => raw_to_rgba(raw),
                Err(raw_err) => {
                    let mut hint = format!("{raw_err}");
                    if hint.contains("Couldn't find camera") {
                        hint = format!(
                            "{hint}. Try converting to DNG (lossless) or using a supported camera profile."
                        );
                    }
                    let libraw_hint = format!("; LibRaw fallback: {libraw_err}");
                    // Try a dummy decode as a last resort (may lack accurate WB/colors but shows pixels)
                    let mut reader = Cursor::new(bytes);
                    decode_dummy(&mut reader)
                        .map_err(|e| {
                            format!(
                                "Failed to decode image: {primary}{libraw_hint}; raw decode: {hint}; dummy decode: {e}"
                            )
                        })
                        .and_then(raw_to_rgba)
                }
            }
        }
    }
}

fn normalize_sample(val: f32, black: f32, white: f32) -> f32 {
    if white <= black {
        return 0.0;
    }
    ((val - black) / (white - black)).clamp(0.0, 1.0)
}

fn raw_to_rgba(raw: RawImage) -> Result<DynamicImage, String> {
    let w = raw.width as u32;
    let h = raw.height as u32;

    // Normalize per-channel black/white (RGBE order, but we map 0->R,1->G,2->B)
    let mut channel_black = [0.0f32; 3];
    let mut channel_white = [65535.0f32; 3];
    for i in 0..3 {
        channel_black[i] = raw.blacklevels.get(i).copied().unwrap_or(0) as f32;
        channel_white[i] = raw.whitelevels.get(i).copied().unwrap_or(65535) as f32;
    }

    // If cpp==3, treat as already-RGB
    if raw.cpp == 3 {
        let mut rgba = RgbaImage::new(w, h);
        match raw.data {
            RawImageData::Integer(data) => {
                for (idx, pixel) in rgba.pixels_mut().enumerate() {
                    let base = idx * 3;
                    let r = normalize_sample(
                        data.get(base).copied().unwrap_or(0) as f32,
                        channel_black[0],
                        channel_white[0],
                    );
                    let g = normalize_sample(
                        data.get(base + 1).copied().unwrap_or(0) as f32,
                        channel_black[1],
                        channel_white[1],
                    );
                    let b = normalize_sample(
                        data.get(base + 2).copied().unwrap_or(0) as f32,
                        channel_black[2],
                        channel_white[2],
                    );
                    *pixel = Rgba([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
                }
            }
            RawImageData::Float(data) => {
                for (idx, pixel) in rgba.pixels_mut().enumerate() {
                    let base = idx * 3;
                    let r = normalize_sample(
                        data.get(base).copied().unwrap_or(0.0),
                        channel_black[0],
                        channel_white[0],
                    );
                    let g = normalize_sample(
                        data.get(base + 1).copied().unwrap_or(0.0),
                        channel_black[1],
                        channel_white[1],
                    );
                    let b = normalize_sample(
                        data.get(base + 2).copied().unwrap_or(0.0),
                        channel_black[2],
                        channel_white[2],
                    );
                    *pixel = Rgba([(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255]);
                }
            }
        }
        return Ok(DynamicImage::ImageRgba8(rgba));
    }

    // Simple Bayer-ish demosaic: accumulate channels then fill missing with neighborhood average
    let len = (w as usize) * (h as usize);
    let mut r = vec![0f32; len];
    let mut g = vec![0f32; len];
    let mut b = vec![0f32; len];
    let mut r_mask = vec![false; len];
    let mut g_mask = vec![false; len];
    let mut b_mask = vec![false; len];

    let get_val = |idx: usize, data_int: Option<&Vec<u16>>, data_f: Option<&Vec<f32>>| -> f32 {
        if let Some(d) = data_int {
            d.get(idx).copied().unwrap_or(0) as f32
        } else if let Some(d) = data_f {
            d.get(idx).copied().unwrap_or(0.0)
        } else {
            0.0
        }
    };

    match &raw.data {
        RawImageData::Integer(data) => {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let idx = y * w as usize + x;
                    let val = get_val(idx, Some(data), None);
                    let color = raw.cfa.color_at(y, x);
                    match color {
                        0 => {
                            r[idx] = normalize_sample(val, channel_black[0], channel_white[0]);
                            r_mask[idx] = true;
                        }
                        1 => {
                            g[idx] = normalize_sample(val, channel_black[1], channel_white[1]);
                            g_mask[idx] = true;
                        }
                        2 => {
                            b[idx] = normalize_sample(val, channel_black[2], channel_white[2]);
                            b_mask[idx] = true;
                        }
                        _ => {
                            g[idx] = normalize_sample(val, channel_black[1], channel_white[1]);
                            g_mask[idx] = true;
                        }
                    }
                }
            }
        }
        RawImageData::Float(data) => {
            for y in 0..h as usize {
                for x in 0..w as usize {
                    let idx = y * w as usize + x;
                    let val = get_val(idx, None, Some(data));
                    let color = raw.cfa.color_at(y, x);
                    match color {
                        0 => {
                            r[idx] = normalize_sample(val, channel_black[0], channel_white[0]);
                            r_mask[idx] = true;
                        }
                        1 => {
                            g[idx] = normalize_sample(val, channel_black[1], channel_white[1]);
                            g_mask[idx] = true;
                        }
                        2 => {
                            b[idx] = normalize_sample(val, channel_black[2], channel_white[2]);
                            b_mask[idx] = true;
                        }
                        _ => {
                            g[idx] = normalize_sample(val, channel_black[1], channel_white[1]);
                            g_mask[idx] = true;
                        }
                    }
                }
            }
        }
    }

    let neighbors = |x: usize, y: usize, w: usize, h: usize| {
        let mut out = Vec::with_capacity(9);
        let xi = x as isize;
        let yi = y as isize;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = xi + dx;
                let ny = yi + dy;
                if nx >= 0 && ny >= 0 && (nx as usize) < w && (ny as usize) < h {
                    out.push((nx as usize, ny as usize));
                }
            }
        }
        out
    };

    let fill_channel = |chan: &mut [f32], mask: &[bool]| {
        let (w_usize, h_usize) = (w as usize, h as usize);
        let mut out = chan.to_vec();
        for y in 0..h_usize {
            for x in 0..w_usize {
                let idx = y * w_usize + x;
                if mask[idx] {
                    continue;
                }
                let mut sum = 0.0;
                let mut count = 0.0;
                for (nx, ny) in neighbors(x, y, w_usize, h_usize) {
                    let nidx = ny * w_usize + nx;
                    if mask[nidx] {
                        sum += chan[nidx];
                        count += 1.0;
                    }
                }
                if count > 0.0 {
                    out[idx] = sum / count;
                } else {
                    out[idx] = chan[idx];
                }
            }
        }
        out
    };

    let r_filled = fill_channel(&mut r, &r_mask);
    let g_filled = fill_channel(&mut g, &g_mask);
    let b_filled = fill_channel(&mut b, &b_mask);

    let mut rgba = RgbaImage::new(w, h);
    for (idx, pixel) in rgba.pixels_mut().enumerate() {
        let rr = r_filled[idx].clamp(0.0, 1.0);
        let gg = g_filled[idx].clamp(0.0, 1.0);
        let bb = b_filled[idx].clamp(0.0, 1.0);
        *pixel = Rgba([
            (rr * 255.0) as u8,
            (gg * 255.0) as u8,
            (bb * 255.0) as u8,
            255,
        ]);
    }

    Ok(DynamicImage::ImageRgba8(rgba))
}

fn placeholder_image() -> DynamicImage {
    let mut img = DynamicImage::new_rgba8(480, 320);
    for (x, y, pixel) in img.as_mut_rgba8().unwrap().enumerate_pixels_mut() {
        let t = (x as f32) / 480.0;
        let b = (y as f32) / 320.0;
        let r = (220.0 - 40.0 * b) as u8;
        let g = (230.0 - 60.0 * t) as u8;
        let bl = (245.0 - 80.0 * (t * b)) as u8;
        *pixel = Rgba([r, g, bl, 255]);
    }
    img
}

fn placeholder_rgba() -> RgbaImage {
    placeholder_image().to_rgba8()
}

fn write_png_to_path(img: &RgbaImage, path: &Path) -> Result<Vec<u8>, String> {
    let buffer = encode_png_fast(img)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, &buffer).map_err(|e| e.to_string())?;
    Ok(buffer)
}

fn render_resized(path: &Path, max_dimension: u32) -> Result<RgbaImage, String> {
    let target = max_dimension.max(1);
    let img = load_dynamic_image(path)?;
    let rgba = img.to_rgba8();
    let source_max = rgba.width().max(rgba.height()).max(1);
    let clamped_target = target.min(source_max);
    Ok(resize_rgba_preserve_aspect(&rgba, clamped_target))
}

/// Clear all in-memory preview caches (masters, scaled variants, LRU list).
pub fn clear_preview_cache() {
    PREVIEW_MASTERS.clear();
    PREVIEW_VARIANTS.clear();
    if let Ok(mut lru) = PREVIEW_LRU.lock() {
        lru.clear();
    }
}

pub fn load_or_create_thumbnail(asset_id: &str, path: &Path) -> Result<Vec<u8>, String> {
    let dir = thumbnails_dir()?;
    let thumb_path = cached_path(&dir, asset_id, "png");
    if thumb_path.exists() {
        return fs::read(&thumb_path).map_err(|e| e.to_string());
    }

    let img = render_resized(path, 360).unwrap_or_else(|_| {
        let ph = placeholder_rgba();
        resize_rgba_preserve_aspect(&ph, 360)
    });
    write_png_to_path(&img, &thumb_path)
}

fn apply_globals_in_place(data: &mut [u8], globals: &GlobalAdjustments) {
    let exposure_mul = 2f32.powf(globals.exposure_ev);
    let contrast = globals.contrast / 100.0;
    let highlights = globals.highlights / 100.0;
    let shadows = globals.shadows / 100.0;
    let whites = globals.whites / 100.0;
    let blacks = globals.blacks / 100.0;
    let vibrance = globals.vibrance / 100.0;
    let saturation = globals.saturation / 100.0;
    let temp = globals.temp / 100.0; // -1..1 approx
    let tint = globals.tint / 100.0; // -1..1 approx

    data.par_chunks_mut(4).for_each(|px| {
        let mut c = [
            px[0] as f32 / 255.0,
            px[1] as f32 / 255.0,
            px[2] as f32 / 255.0,
            px[3] as f32 / 255.0,
        ];
        let a = c[3];

        for i in 0..3 {
            c[i] *= exposure_mul;
        }
        c[0] *= 1.0 + temp * 0.5 + tint * 0.2;
        c[2] *= 1.0 - temp * 0.5 + tint * 0.2;
        c[1] *= 1.0 - tint * 0.2;

        let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];

        let highlights_mask = (l - 0.5).max(0.0f32) * 2.0;
        let shadows_mask = (0.5 - l).max(0.0f32) * 2.0;
        for i in 0..3 {
            c[i] *= 1.0 + highlights * highlights_mask;
            c[i] *= 1.0 + shadows * shadows_mask;
        }

        for i in 0..3 {
            c[i] = c[i] + whites * 0.1;
            c[i] = c[i] - blacks * 0.1;
        }

        for i in 0..3 {
            c[i] = (c[i] - 0.5) * (1.0 + contrast) + 0.5;
        }

        let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let sat_factor = 1.0 + saturation;
        let vib_mask = (1.0 - ((c[0] - l).abs() + (c[1] - l).abs() + (c[2] - l).abs()) / 3.0)
            .clamp(0.0f32, 1.0);
        let vib_factor = 1.0 + vibrance * vib_mask;
        for i in 0..3 {
            c[i] = l + (c[i] - l) * sat_factor * vib_factor;
        }

        for i in 0..3 {
            c[i] = c[i].clamp(0.0, 1.0);
        }

        px[0] = (c[0] * 255.0).round() as u8;
        px[1] = (c[1] * 255.0).round() as u8;
        px[2] = (c[2] * 255.0).round() as u8;
        px[3] = (a * 255.0).round() as u8;
    });
}

fn globals_are_identity(globals: &GlobalAdjustments) -> bool {
    let eps = 1e-4;
    globals.exposure_ev.abs() < eps
        && globals.contrast.abs() < eps
        && globals.highlights.abs() < eps
        && globals.shadows.abs() < eps
        && globals.whites.abs() < eps
        && globals.blacks.abs() < eps
        && globals.temp.abs() < eps
        && globals.tint.abs() < eps
        && globals.vibrance.abs() < eps
        && globals.saturation.abs() < eps
}

fn layers_have_effect(layers: &[AdjustmentLayer]) -> bool {
    layers
        .iter()
        .any(|layer| layer.enabled && layer.opacity > 0.0)
}

fn apply_local_layer_in_place(data: &mut [u8], w: u32, h: u32, layer: &AdjustmentLayer) {
    if !layer.enabled || layer.opacity <= 0.0 {
        return;
    }
    let start = layer.mask.start;
    let end = layer.mask.end;
    let feather = layer.mask.feather.max(0.001);
    let invert = layer.mask.invert;
    let opacity = layer.opacity;
    let adj = &layer.adjustments;

    let temp = adj.temp / 100.0;
    let tint = adj.tint / 100.0;
    let exposure_mul = 2f32.powf(adj.exposure_ev);
    let saturation = adj.saturation / 100.0;

    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let len_sq = (dx * dx + dy * dy).max(1e-6);

    data.par_chunks_mut(4).enumerate().for_each(|(idx, px)| {
        let x = (idx as u32 % w) as f32 / w as f32;
        let y = (idx as u32 / w) as f32 / h as f32;

        let pxv = x - start.0;
        let pyv = y - start.1;
        let t = (pxv * dx + pyv * dy) / len_sq;
        let t_clamped = t.clamp(0.0, 1.0);

        let edge0 = 0.5 - feather * 0.5;
        let edge1 = 0.5 + feather * 0.5;
        let mut mask = ((t_clamped - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
        mask = mask * mask * (3.0 - 2.0 * mask);
        if invert {
            mask = 1.0 - mask;
        }
        mask *= opacity;
        if mask <= 0.0001 {
            return;
        }

        let mut c = [
            px[0] as f32 / 255.0,
            px[1] as f32 / 255.0,
            px[2] as f32 / 255.0,
            px[3] as f32 / 255.0,
        ];

        for i in 0..3 {
            c[i] *= exposure_mul;
        }
        c[0] *= 1.0 + temp * 0.5 + tint * 0.2;
        c[2] *= 1.0 - temp * 0.5 + tint * 0.2;
        c[1] *= 1.0 - tint * 0.2;

        let l = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
        let sat_factor = 1.0 + saturation;
        for i in 0..3 {
            c[i] = l + (c[i] - l) * sat_factor;
        }
        for i in 0..3 {
            c[i] = c[i].clamp(0.0, 1.0);
        }

        px[0] = ((px[0] as f32 / 255.0 * (1.0 - mask) + c[0] * mask) * 255.0).round() as u8;
        px[1] = ((px[1] as f32 / 255.0 * (1.0 - mask) + c[1] * mask) * 255.0).round() as u8;
        px[2] = ((px[2] as f32 / 255.0 * (1.0 - mask) + c[2] * mask) * 255.0).round() as u8;
    });
}

fn apply_layers_in_place(data: &mut [u8], w: u32, h: u32, layers: &[AdjustmentLayer]) {
    if layers.is_empty() {
        return;
    }
    layers
        .iter()
        .for_each(|layer| apply_local_layer_in_place(data, w, h, layer));
}

fn encode_png_fast(img: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut buffer = Vec::new();
    let cursor = Cursor::new(&mut buffer);
    let encoder = PngEncoder::new_with_quality(cursor, CompressionType::Fast, FilterType::NoFilter);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ColorType::Rgba8.into(),
        )
        .map_err(|e| format!("Failed to encode PNG: {e}"))?;
    Ok(buffer)
}

pub fn render_preview_with_recipe(
    asset_id: &str,
    path: &Path,
    recipe: Option<EditRecipe>,
    max_dimension: Option<u32>,
) -> Result<Vec<u8>, String> {
    let target = max_dimension.unwrap_or(1440);
    let base = scaled_preview(asset_id, path, target)?;
    let mut working: RgbaImage = (*base).clone();

    if let Some(r) = recipe.as_ref() {
        if !globals_are_identity(&r.globals) {
            if let Some(gpu_img) = gpu::apply_globals_rgba(&working, &r.globals) {
                working = gpu_img;
            } else {
                apply_globals_in_place(working.as_mut(), &r.globals);
            }
        }
        if layers_have_effect(&r.layers) {
            let (w, h) = working.dimensions();
            apply_layers_in_place(working.as_mut(), w, h, &r.layers);
        }
    }

    encode_png_fast(&working)
}
