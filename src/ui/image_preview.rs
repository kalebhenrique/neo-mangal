use image::imageops::FilterType;
use image::GenericImageView;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::path::Path;

pub struct ImagePreview;

impl ImagePreview {
    /// Renders an image file into Ratatui Text Lines using Unicode Halfblocks (▀)
    /// Each character cell represents 2 vertical pixels (foreground = top, background = bottom).
    /// Dynamically scales to fit the available terminal area (`max_cols` x `max_rows`)
    /// based on the terminal window size, preserving aspect ratio and occupying the preview section.
    pub fn render_halfblocks(
        path: &Path,
        max_cols: u16,
        max_rows: u16,
    ) -> Option<Vec<Line<'static>>> {
        let img = image::open(path).ok()?;
        let (orig_w, orig_h) = img.dimensions();
        if orig_w == 0 || orig_h == 0 || max_cols == 0 || max_rows == 0 {
            return None;
        }

        // Terminal cells are roughly 1:2 aspect ratio (width:height).
        // A block '▀' has 1 cell width and 2 vertical subpixels, making each subpixel roughly 1:1 square.
        let pixel_w = orig_w as f32;
        let pixel_h = (orig_h as f32) / 2.0; // In terminal cell units

        let scale_w = (max_cols as f32) / pixel_w;
        let scale_h = (max_rows as f32) / pixel_h;
        // Scale to occupy the preview section to the maximum bounds without distortion
        let scale = scale_w.min(scale_h);

        let target_cols = ((pixel_w * scale).round() as u32).clamp(1, max_cols as u32);
        let target_rows = ((pixel_h * scale).round() as u32).clamp(1, max_rows as u32);

        let resized = img.resize_exact(target_cols, target_rows * 2, FilterType::Lanczos3);
        let rgb_img = resized.to_rgba8();

        // Calculate vertical centering padding if target_rows is smaller than max_rows
        let top_padding = (max_rows.saturating_sub(target_rows as u16)) / 2;
        let mut lines = Vec::with_capacity((target_rows as usize) + (top_padding as usize));

        for _ in 0..top_padding {
            lines.push(Line::from(""));
        }

        for row in 0..target_rows {
            let mut spans = Vec::with_capacity(target_cols as usize);
            for col in 0..target_cols {
                let top_pixel = rgb_img.get_pixel(col, row * 2);
                let bot_pixel = rgb_img.get_pixel(col, row * 2 + 1);

                // Check alpha
                let fg_color = if top_pixel[3] < 128 {
                    Color::Reset
                } else {
                    Color::Rgb(top_pixel[0], top_pixel[1], top_pixel[2])
                };

                let bg_color = if bot_pixel[3] < 128 {
                    Color::Reset
                } else {
                    Color::Rgb(bot_pixel[0], bot_pixel[1], bot_pixel[2])
                };

                spans.push(Span::styled(
                    "▀",
                    Style::default().fg(fg_color).bg(bg_color),
                ));
            }
            lines.push(Line::from(spans));
        }

        Some(lines)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_render_halfblocks_dummy_image() {
        let temp_path = std::env::temp_dir().join("neo_mangal_test_preview.png");
        let mut img = RgbImage::new(4, 4);
        img.put_pixel(0, 0, Rgb([255, 0, 0]));
        img.put_pixel(0, 1, Rgb([0, 255, 0]));
        img.save(&temp_path).unwrap();

        let lines = ImagePreview::render_halfblocks(&temp_path, 10, 5);
        assert!(lines.is_some());
        let l = lines.unwrap();
        assert!(!l.is_empty());

        // Test with zero dimensions returns None
        assert!(ImagePreview::render_halfblocks(&temp_path, 0, 5).is_none());
        assert!(ImagePreview::render_halfblocks(&temp_path, 10, 0).is_none());

        // Test dynamic scaling to larger preview bounds (e.g. 50x30)
        let large_preview = ImagePreview::render_halfblocks(&temp_path, 50, 30);
        assert!(large_preview.is_some());
        let lines_large = large_preview.unwrap();
        assert!(lines_large.len() <= 30);

        let _ = std::fs::remove_file(&temp_path);
    }
}
