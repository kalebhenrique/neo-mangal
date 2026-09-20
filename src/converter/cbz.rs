use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use crate::error::{NeoError, Result};

pub struct CbzPacker;

impl CbzPacker {
    /// Injects a custom cover if provided (copied as 0000_cover.<ext>)
    /// and packages all images in `source_dir` into `target_cbz`.
    pub fn package(
        source_dir: &Path,
        custom_cover: Option<&Path>,
        target_cbz: &Path,
    ) -> Result<PathBuf> {
        if !source_dir.is_dir() {
            return Err(NeoError::Other(format!(
                "Source directory does not exist: {}",
                source_dir.display()
            )));
        }

        // 1. Process custom cover if provided
        if let Some(cover_path) = custom_cover {
            if cover_path.is_file() {
                let extension = cover_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("jpg");
                let destination = source_dir.join(format!("0000_cover.{}", extension));
                fs::copy(cover_path, &destination)?;
            } else {
                return Err(NeoError::Other(format!(
                    "Custom cover file not found: {}",
                    cover_path.display()
                )));
            }
        }

        // 2. Collect image files in directory
        let mut image_entries: Vec<PathBuf> = WalkDir::new(source_dir)
            .max_depth(1)
            .into_iter()
            .flatten()
            .filter(|e| e.file_type().is_file())
            .map(|e| e.into_path())
            .filter(|path| {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    matches!(
                        ext.to_lowercase().as_str(),
                        "jpg" | "jpeg" | "png" | "webp" | "avif" | "gif"
                    )
                } else {
                    false
                }
            })
            .collect();

        if image_entries.is_empty() {
            return Err(NeoError::Other(format!(
                "No images found in directory: {}",
                source_dir.display()
            )));
        }

        // 3. Sort entries alphabetically so 0000_cover is guaranteed to be first
        image_entries.sort();

        // 4. Ensure target directory exists
        if let Some(parent) = target_cbz.parent() {
            fs::create_dir_all(parent)?;
        }

        // 5. Create CBZ zip archive
        let file = File::create(target_cbz)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        let mut buffer = Vec::new();
        for image_path in &image_entries {
            let filename = image_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("image.jpg");

            zip.start_file(filename, options)?;
            let mut f = File::open(image_path)?;
            buffer.clear();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }

        zip.finish()?;
        Ok(target_cbz.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_images_into_cbz() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_test_cbz");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Create dummy pages
        fs::write(temp_dir.join("0002.png"), b"page 2").unwrap();
        fs::write(temp_dir.join("0001.png"), b"page 1").unwrap();

        // Create dummy custom cover
        let cover_path = temp_dir.join("my_cover.jpg");
        fs::write(&cover_path, b"my custom cover").unwrap();

        let cbz_path = temp_dir.join("output.cbz");
        let result = CbzPacker::package(&temp_dir, Some(&cover_path), &cbz_path);
        assert!(result.is_ok());
        assert!(cbz_path.exists());

        // Check if 0000_cover.jpg was created
        assert!(temp_dir.join("0000_cover.jpg").exists());

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
