use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::models::Metadata;
use exif;

pub fn read_metadata(path: &Path) -> Result<Metadata, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut bufreader = BufReader::new(file);
    let exifreader = exif::Reader::new();
    let exif = exifreader
        .read_from_container(&mut bufreader)
        .map_err(|e| format!("EXIF read error: {e}"))?;

    let mut meta = Metadata::default();

    for field in exif.fields() {
        match field.tag {
            exif::Tag::Model => {
                meta.camera = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::LensModel => {
                meta.lens = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::ISOSpeed => {
                meta.iso = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::ExposureTime => {
                meta.shutter = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::FNumber => {
                meta.aperture = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::FocalLength => {
                meta.focal = Some(field.display_value().with_unit(&exif).to_string())
            }
            exif::Tag::DateTimeOriginal => {
                meta.date = Some(field.display_value().with_unit(&exif).to_string())
            }
            _ => {}
        }
    }

    Ok(meta)
}
