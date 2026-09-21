use std::fs::{self, File};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use image::GenericImageView;
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref};
use walkdir::WalkDir;
use crate::error::{NeoError, Result};

pub struct PdfPacker;

impl PdfPacker {
    /// Injects a custom cover if provided (copied as 0000_cover.<ext>)
    /// and packages all images in `source_dir` into `target_pdf`.
    pub fn package(
        source_dir: &Path,
        custom_cover: Option<&Path>,
        target_pdf: &Path,
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

        // 4. Ensure parent directory exists
        if let Some(parent) = target_pdf.parent() {
            fs::create_dir_all(parent)?;
        }

        // 5. Build PDF with pdf-writer
        let mut pdf = Pdf::new();

        // Object ID allocation
        let mut next_id = 1;
        let mut alloc_id = || {
            let id = Ref::new(next_id);
            next_id += 1;
            id
        };

        let catalog_id = alloc_id();
        let page_tree_id = alloc_id();

        let mut page_ids = Vec::new();

        for img_path in &image_entries {
            let page_id = alloc_id();
            let content_id = alloc_id();
            let image_id = alloc_id();
            page_ids.push(page_id);

            // Read image and get JPEG bytes + dimensions
            let (jpeg_data, width, height) = Self::load_as_jpeg(img_path)?;

            // Add Image XObject
            let image_name = Name(b"Im0");
            let mut image = pdf.image_xobject(image_id, &jpeg_data);
            image.width(width as i32);
            image.height(height as i32);
            image.color_space().device_rgb();
            image.bits_per_component(8);
            image.filter(Filter::DctDecode);
            image.finish();

            // Content stream: scale image to full page size (width x height pt)
            let mut content = Content::new();
            content.save_state();
            content.transform([width as f32, 0.0, 0.0, height as f32, 0.0, 0.0]);
            content.x_object(image_name);
            content.restore_state();

            pdf.stream(content_id, &content.finish());

            // Page object
            let mut page = pdf.page(page_id);
            page.parent(page_tree_id);
            page.media_box(Rect::new(0.0, 0.0, width as f32, height as f32));
            page.contents(content_id);
            page.resources().x_objects().pair(image_name, image_id);
            page.finish();
        }

        // Pages root
        pdf.pages(page_tree_id)
            .kids(page_ids.iter().copied())
            .count(page_ids.len() as i32);

        // Catalog root
        pdf.catalog(catalog_id).pages(page_tree_id);

        let pdf_bytes = pdf.finish();
        let mut out_file = File::create(target_pdf)?;
        out_file.write_all(&pdf_bytes)?;

        Ok(target_pdf.to_path_buf())
    }

    /// Loads image from path, ensuring it is converted to JPEG bytes if needed.
    /// Uses magic bytes inspection instead of relying on file extension,
    /// because scrapers often save JPEGs or WebPs with a .png extension.
    fn load_as_jpeg(path: &Path) -> Result<(Vec<u8>, u32, u32)> {
        let data = fs::read(path)?;
        if data.is_empty() {
            return Err(NeoError::Other(format!("Image file is empty: {}", path.display())));
        }

        // 1. If it's already a valid JPEG (starts with 0xFF, 0xD8, 0xFF), embed directly!
        if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
            // Fast dimension extraction without full pixel decoding
            if let Ok(reader) = image::ImageReader::new(Cursor::new(&data)).with_guessed_format() {
                if let Ok((w, h)) = reader.into_dimensions() {
                    return Ok((data, w, h));
                }
            }
            // Fallback for dimensions
            if let Ok(img) = image::load_from_memory(&data) {
                return Ok((data, img.width(), img.height()));
            }
        }

        // 2. Decode any format (PNG, WebP, GIF, AVIF, etc.) by inspecting content magic bytes
        let img = image::load_from_memory(&data).map_err(|e| {
            NeoError::Other(format!("Failed to decode image {}: {}", path.display(), e))
        })?;

        let (w, h) = img.dimensions();
        let rgb_img = img.to_rgb8();

        let mut jpeg_buf = Cursor::new(Vec::new());
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut jpeg_buf, 92);
        encoder.encode(rgb_img.as_raw(), w, h, image::ExtendedColorType::Rgb8).map_err(|e| {
            NeoError::Other(format!("Failed to encode JPEG for {}: {}", path.display(), e))
        })?;

        Ok((jpeg_buf.into_inner(), w, h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::DynamicImage;

    #[test]
    fn test_pdf_packer() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_test_pdf");
        let _ = fs::remove_dir_all(&temp_dir);
        let src = temp_dir.join("manga_src");
        fs::create_dir_all(&src).unwrap();

        // Create a dummy image
        let dummy = DynamicImage::new_rgb8(100, 200);
        let img_path = src.join("01.jpg");
        dummy.save(&img_path).unwrap();

        let target_pdf = temp_dir.join("output.pdf");
        let res = PdfPacker::package(&src, None, &target_pdf);
        assert!(res.is_ok());
        assert!(target_pdf.exists());
        assert!(fs::metadata(&target_pdf).unwrap().len() > 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_pdf_packer_with_mismatched_extensions() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_test_pdf_mismatched");
        let _ = fs::remove_dir_all(&temp_dir);
        let src = temp_dir.join("manga_src");
        fs::create_dir_all(&src).unwrap();

        // Create a real JPEG image, but name it .png (exactly what scrapers do!)
        let dummy = DynamicImage::new_rgb8(100, 200);
        let img_path = src.join("0000.png");
        let mut jpeg_bytes = Vec::new();
        let mut encoder = image::codecs::jpeg::JpegEncoder::new(&mut jpeg_bytes);
        encoder.encode(dummy.to_rgb8().as_raw(), 100, 200, image::ExtendedColorType::Rgb8).unwrap();
        fs::write(&img_path, &jpeg_bytes).unwrap();

        let target_pdf = temp_dir.join("output.pdf");
        let res = PdfPacker::package(&src, None, &target_pdf);
        assert!(res.is_ok(), "Failed to package PDF with mismatched extension: {:?}", res.err());
        assert!(target_pdf.exists());
        assert!(fs::metadata(&target_pdf).unwrap().len() > 0);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
