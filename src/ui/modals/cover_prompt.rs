use crate::i18n::AppLanguage;
use crate::ui::image_preview::ImagePreview;
use crate::ui::modals::settings::centered_rect;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::PathBuf;

pub struct ProcessModal {
    pub input_path: String,
    pub target_title: String,
    pub chapters_count: usize,
    pub format: String,
    pub convert_kcc: bool,
    pub fuse_volume: bool,
    pub cached_preview: Option<Vec<Line<'static>>>,
    last_preview_path: Option<String>,
    last_preview_dims: Option<(u16, u16)>,
}

impl ProcessModal {
    pub fn new(
        target_title: &str,
        chapters_count: usize,
        default_format: &str,
        default_kcc: bool,
    ) -> Self {
        let fmt = if default_format.trim().is_empty() {
            "AZW3".to_string()
        } else {
            default_format.to_uppercase()
        };
        let is_kcc = default_kcc && fmt != "CBZ" && fmt != "PDF";
        Self {
            input_path: String::new(),
            target_title: target_title.to_string(),
            chapters_count,
            format: fmt,
            convert_kcc: is_kcc,
            fuse_volume: chapters_count > 1, // Default fusion to true when multiple chapters
            cached_preview: None,
            last_preview_path: None,
            last_preview_dims: None,
        }
    }

    pub fn is_kcc(&self) -> bool {
        self.convert_kcc
            && !self.format.eq_ignore_ascii_case("CBZ")
            && !self.format.eq_ignore_ascii_case("PDF")
    }

    pub fn cycle_format(&mut self) {
        let formats = ["AZW3", "CBZ", "EPUB", "MOBI", "PDF"];
        if let Some(pos) = formats
            .iter()
            .position(|&f| f.eq_ignore_ascii_case(&self.format))
        {
            let next_idx = (pos + 1) % formats.len();
            self.format = formats[next_idx].to_string();
        } else {
            self.format = "AZW3".to_string();
        }
        self.convert_kcc =
            !self.format.eq_ignore_ascii_case("CBZ") && !self.format.eq_ignore_ascii_case("PDF");
    }

    pub fn handle_char(&mut self, c: char) {
        self.input_path.push(c);
        self.invalidate_preview();
    }

    pub fn handle_backspace(&mut self) {
        self.input_path.pop();
        self.invalidate_preview();
    }

    pub fn handle_paste(&mut self, text: &str) {
        let cleaned = Self::clean_drag_drop_path(text);
        self.input_path = cleaned;
        self.invalidate_preview();
    }

    fn invalidate_preview(&mut self) {
        let validated = self.get_validated_path();
        if validated.is_none() {
            self.cached_preview = None;
            self.last_preview_path = None;
            self.last_preview_dims = None;
        } else {
            // Signal path change so next render updates preview to match current container dimensions
            self.last_preview_path = None;
        }
    }

    pub fn toggle_kcc(&mut self) {
        if self.is_kcc() {
            self.format = "CBZ".to_string();
            self.convert_kcc = false;
        } else {
            self.format = "AZW3".to_string();
            self.convert_kcc = true;
        }
    }

    pub fn toggle_fusion(&mut self) {
        if self.chapters_count > 1 {
            self.fuse_volume = !self.fuse_volume;
        }
    }

    /// Dynamically updates the cover image preview to occupy the preview area (`max_cols` x `max_rows`)
    /// based on the terminal window size. Caches results to avoid redundant disk I/O and resizing.
    pub fn update_preview(&mut self, max_cols: u16, max_rows: u16) {
        let validated = self.get_validated_path();
        let path_str = validated.as_ref().map(|p| p.to_string_lossy().to_string());

        if path_str == self.last_preview_path
            && self.last_preview_dims == Some((max_cols, max_rows))
        {
            return;
        }

        self.last_preview_path = path_str;
        self.last_preview_dims = Some((max_cols, max_rows));

        if let Some(path) = validated {
            self.cached_preview = ImagePreview::render_halfblocks(&path, max_cols, max_rows);
        } else {
            self.cached_preview = None;
        }
    }

    pub fn clean_drag_drop_path(input: &str) -> String {
        let mut text = input.trim();

        if let Some(stripped) = text.strip_prefix("file://") {
            text = stripped;
        }

        if (text.starts_with('\'') && text.ends_with('\''))
            || (text.starts_with('"') && text.ends_with('"'))
        {
            if text.len() >= 2 {
                text = &text[1..text.len() - 1];
            }
        }

        let unescaped = text.replace(r"\ ", " ");

        if unescaped.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(&unescaped[2..]).to_string_lossy().to_string();
            }
        }

        unescaped.trim().to_string()
    }

    pub fn get_validated_path(&self) -> Option<PathBuf> {
        let cleaned = Self::clean_drag_drop_path(&self.input_path);
        if cleaned.is_empty() {
            return None;
        }

        let path = PathBuf::from(cleaned);
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let lower = ext.to_lowercase();
                if matches!(lower.as_str(), "jpg" | "jpeg" | "png" | "webp" | "avif") {
                    return Some(path);
                }
            }
        }
        None
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let modal_area = centered_rect(86, 82, area);
        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰂬 Download & Process Options ",
            AppLanguage::Portuguese => " 󰂬 Opções de Download e Processamento ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        // Split into Left (Options & Inputs) and Right (Image Preview)
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([
                Constraint::Percentage(54), // Inputs & controls
                Constraint::Percentage(46), // Cover Preview panel (dynamically scales to terminal window)
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // --- LEFT PANEL ---
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2), // Target chapters header
                Constraint::Length(3), // Drag & drop cover input
                Constraint::Length(1), // Validation feedback
                Constraint::Length(5), // Checkboxes (KCC & Fusion)
                Constraint::Length(3), // Warning if not fusion
                Constraint::Min(2),    // Action buttons
            ])
            .split(main_chunks[0]);

        // 1. Header line
        let target_info = match lang {
            AppLanguage::English => format!(
                "󰉋 Target: {} ({} chapter(s))",
                self.target_title, self.chapters_count
            ),
            AppLanguage::Portuguese => format!(
                "󰉋 Alvo: {} ({} capítulo(s))",
                self.target_title, self.chapters_count
            ),
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![Span::styled(
                target_info,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )])),
            left_chunks[0],
        );

        // 2. Cover input box
        let cover_title = match lang {
            AppLanguage::English => " 󰋩 Custom Cover (Drag & Drop or Type Path) ",
            AppLanguage::Portuguese => " 󰋩 Capa Customizada (Arraste ou Digite o Caminho) ",
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(cover_title)
            .border_style(Style::default().fg(Color::Magenta));

        let display_path = if self.input_path.is_empty() {
            let placeholder = match lang {
                AppLanguage::English => "Drop cover image here (optional)...",
                AppLanguage::Portuguese => "Solte a imagem da capa aqui (opcional)...",
            };
            Line::from(vec![Span::styled(
                placeholder,
                Style::default().fg(Color::DarkGray),
            )])
        } else {
            Line::from(vec![
                Span::styled(&self.input_path, Style::default().fg(Color::White)),
                Span::styled("█", Style::default().fg(Color::Yellow)),
            ])
        };
        frame.render_widget(
            Paragraph::new(display_path).block(input_block),
            left_chunks[1],
        );

        // 3. Validation line
        let val_line = match self.get_validated_path() {
            Some(_) => match lang {
                AppLanguage::English => Line::from(vec![Span::styled(
                    "󰄬 Valid cover image detected",
                    Style::default().fg(Color::Green),
                )]),
                AppLanguage::Portuguese => Line::from(vec![Span::styled(
                    "󰄬 Imagem de capa válida detectada",
                    Style::default().fg(Color::Green),
                )]),
            },
            None if self.input_path.trim().is_empty() => match lang {
                AppLanguage::English => Line::from(vec![Span::styled(
                    "ℹ Using chapter default first page as cover",
                    Style::default().fg(Color::DarkGray),
                )]),
                AppLanguage::Portuguese => Line::from(vec![Span::styled(
                    "ℹ Usando a primeira página padrão como capa",
                    Style::default().fg(Color::DarkGray),
                )]),
            },
            None => match lang {
                AppLanguage::English => Line::from(vec![Span::styled(
                    "󰀦 File not found or invalid format",
                    Style::default().fg(Color::Red),
                )]),
                AppLanguage::Portuguese => Line::from(vec![Span::styled(
                    "󰀦 Arquivo não encontrado ou inválido",
                    Style::default().fg(Color::Red),
                )]),
            },
        };
        frame.render_widget(Paragraph::new(val_line), left_chunks[2]);

        // 4. Format selection [o] & Fusion [f]
        let is_fusion_disabled = self.chapters_count <= 1;

        let format_badge = format!("[ {} ]", self.format);
        let format_label = match lang {
            AppLanguage::English => " Format: ",
            AppLanguage::Portuguese => " Formato: ",
        };
        let cycle_hint = match lang {
            AppLanguage::English => "(press 'o' to cycle: AZW3 / CBZ / EPUB / MOBI / PDF)",
            AppLanguage::Portuguese => "(tecle 'o' para alternar: AZW3 / CBZ / EPUB / MOBI / PDF)",
        };

        let format_desc = match (self.format.as_str(), lang) {
            ("AZW3", AppLanguage::English) => "Kindle KF8 (300 ppi, max quality) via KCC",
            ("AZW3", AppLanguage::Portuguese) => "Kindle KF8 (300 ppi, máxima qualidade) via KCC",
            ("CBZ", AppLanguage::English) => "Direct CBZ archive (no KCC)",
            ("CBZ", AppLanguage::Portuguese) => "Arquivo CBZ direto (sem KCC)",
            ("EPUB", AppLanguage::English) => "Standard EPUB e-book via KCC",
            ("EPUB", AppLanguage::Portuguese) => "Livro digital EPUB padrão via KCC",
            ("MOBI", AppLanguage::English) => "Kindle MOBI legacy dual-format via KCC",
            ("MOBI", AppLanguage::Portuguese) => "Kindle MOBI legado duplo via KCC",
            ("PDF", AppLanguage::English) => "Portable Document Format (no KCC)",
            ("PDF", AppLanguage::Portuguese) => "Documento PDF portátil (sem KCC)",
            _ => "Output archive",
        };

        let fusion_check = if self.fuse_volume && !is_fusion_disabled {
            "󰄲"
        } else {
            "󰄱"
        };

        let fusion_label = match (is_fusion_disabled, lang) {
            (true, AppLanguage::English) => {
                " Volume Fusion: Merge chapters (disabled - only 1 chapter)"
            }
            (true, AppLanguage::Portuguese) => {
                " Fusão de Volume: Unir capítulos (desabilitado - apenas 1 capítulo)"
            }
            (false, AppLanguage::English) => {
                " Volume Fusion: Merge all chapters into 1 volume (press 'f' to toggle)"
            }
            (false, AppLanguage::Portuguese) => {
                " Fusão de Volume: Unir capítulos em 1 volume (tecle 'f' para alternar)"
            }
        };

        let fusion_check_style = if is_fusion_disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        };

        let fusion_text_style = if is_fusion_disabled {
            Style::default().fg(Color::DarkGray)
        } else {
            Style::default().fg(Color::White)
        };

        let chk_lines = vec![
            Line::from(vec![
                Span::styled(
                    " 󰒓",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(format_label, Style::default().fg(Color::White)),
                Span::styled(
                    format_badge,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(cycle_hint, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::raw("    ↳ "),
                Span::styled(format_desc, Style::default().fg(Color::LightCyan)),
            ]),
            Line::from(vec![
                Span::styled(format!(" {} ", fusion_check), fusion_check_style),
                Span::styled(fusion_label, fusion_text_style),
            ]),
        ];
        frame.render_widget(Paragraph::new(chk_lines), left_chunks[3]);

        // 5. Warning when NOT fusing multiple chapters
        let warn_p = if self.chapters_count > 1
            && !self.fuse_volume
            && !self.input_path.trim().is_empty()
        {
            let warn_text = match lang {
                AppLanguage::English => "󰀦 Warning: Without volume fusion, this custom cover will be repeated as the front page for each individual chapter file.",
                AppLanguage::Portuguese => "󰀦 Aviso: Sem a fusão de volume, esta capa customizada será repetida na primeira página de cada capítulo individual.",
            };
            Paragraph::new(Line::from(vec![Span::styled(
                warn_text,
                Style::default().fg(Color::Rgb(255, 170, 0)),
            )]))
        } else {
            Paragraph::new("")
        };
        frame.render_widget(warn_p, left_chunks[4]);

        // 6. Action buttons
        let actions = match (is_fusion_disabled, lang) {
            (false, AppLanguage::English) => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Start    "),
                Span::styled(
                    " [o] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Format    "),
                Span::styled(
                    " [f] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Fusion    "),
                Span::styled(
                    " [Esc] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("Cancel"),
            ],
            (true, AppLanguage::English) => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Start    "),
                Span::styled(
                    " [o] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Format    "),
                Span::styled(" [f] ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Fusion (disabled)    ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    " [Esc] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("Cancel"),
            ],
            (false, AppLanguage::Portuguese) => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Iniciar    "),
                Span::styled(
                    " [o] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Formato    "),
                Span::styled(
                    " [f] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Fusão    "),
                Span::styled(
                    " [Esc] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("Cancelar"),
            ],
            (true, AppLanguage::Portuguese) => vec![
                Span::styled(
                    " [Enter] ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Iniciar    "),
                Span::styled(
                    " [o] ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("Formato    "),
                Span::styled(" [f] ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "Fusão (desabilitado)    ",
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    " [Esc] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::raw("Cancelar"),
            ],
        };
        frame.render_widget(
            Paragraph::new(Line::from(actions)).alignment(Alignment::Center),
            left_chunks[5],
        );

        // --- RIGHT PANEL: COVER PREVIEW ---
        let preview_title = match lang {
            AppLanguage::English => " 󰋩 Cover Preview ",
            AppLanguage::Portuguese => " 󰋩 Prévia da Capa ",
        };

        let preview_block = Block::default()
            .borders(Borders::ALL)
            .title(preview_title)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner_preview = preview_block.inner(main_chunks[1]);
        self.update_preview(inner_preview.width, inner_preview.height);

        if let Some(ref lines) = self.cached_preview {
            let preview_p = Paragraph::new(lines.clone())
                .alignment(Alignment::Center)
                .block(preview_block);
            frame.render_widget(preview_p, main_chunks[1]);
        } else {
            let pad = (inner_preview.height.saturating_sub(2)) / 2;
            let mut empty_text = Vec::with_capacity((pad as usize) + 2);
            for _ in 0..pad {
                empty_text.push(Line::from(""));
            }
            match lang {
                AppLanguage::English => {
                    empty_text.push(Line::from(Span::styled(
                        "No image loaded",
                        Style::default().fg(Color::DarkGray),
                    )));
                    empty_text.push(Line::from(Span::styled(
                        "Drop a file to preview",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                AppLanguage::Portuguese => {
                    empty_text.push(Line::from(Span::styled(
                        "Nenhuma imagem carregada",
                        Style::default().fg(Color::DarkGray),
                    )));
                    empty_text.push(Line::from(Span::styled(
                        "Solte um arquivo para pré-visualizar",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            };
            let preview_p = Paragraph::new(empty_text)
                .alignment(Alignment::Center)
                .block(preview_block);
            frame.render_widget(preview_p, main_chunks[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_drag_drop_path() {
        assert_eq!(
            ProcessModal::clean_drag_drop_path("file:///Users/test/cover.jpg"),
            "/Users/test/cover.jpg"
        );
        assert_eq!(
            ProcessModal::clean_drag_drop_path("'/Users/test/my cover.jpg'"),
            "/Users/test/my cover.jpg"
        );
        assert_eq!(
            ProcessModal::clean_drag_drop_path(r"/Users/test/my\ cover.jpg"),
            "/Users/test/my cover.jpg"
        );
    }

    #[test]
    fn test_process_modal_toggles() {
        let mut modal = ProcessModal::new("Test Manga", 5, "AZW3", true);
        assert!(modal.is_kcc());
        assert!(modal.fuse_volume);
        assert_eq!(modal.format, "AZW3");

        // Cycle format
        modal.cycle_format();
        assert_eq!(modal.format, "CBZ");
        assert!(!modal.is_kcc());

        modal.cycle_format();
        assert_eq!(modal.format, "EPUB");
        assert!(modal.is_kcc());

        modal.cycle_format();
        assert_eq!(modal.format, "MOBI");
        assert!(modal.is_kcc());

        modal.cycle_format();
        assert_eq!(modal.format, "PDF");
        assert!(!modal.is_kcc());

        modal.cycle_format();
        assert_eq!(modal.format, "AZW3");
        assert!(modal.is_kcc());

        // Toggle KCC (switches between CBZ and AZW3)
        modal.toggle_kcc();
        assert_eq!(modal.format, "CBZ");
        assert!(!modal.is_kcc());

        modal.toggle_kcc();
        assert_eq!(modal.format, "AZW3");
        assert!(modal.is_kcc());

        modal.toggle_fusion();
        assert!(!modal.fuse_volume);

        // For single chapter, fusion defaults to false and cannot be toggled on
        let mut single_modal = ProcessModal::new("Chapter 1", 1, "CBZ", false);
        assert!(!single_modal.fuse_volume);
        single_modal.toggle_fusion();
        assert!(!single_modal.fuse_volume);
    }

    #[test]
    fn test_process_modal_dynamic_preview() {
        let mut modal = ProcessModal::new("Test", 1, "AZW3", true);
        assert!(modal.cached_preview.is_none());

        // Invalid path keeps preview None
        modal.input_path = "/path/does/not/exist.png".to_string();
        modal.update_preview(40, 20);
        assert!(modal.cached_preview.is_none());

        // Create temporary image
        let temp_path = std::env::temp_dir().join("neo_mangal_modal_test.png");
        let img = image::RgbImage::new(10, 10);
        img.save(&temp_path).unwrap();

        modal.input_path = temp_path.to_string_lossy().to_string();
        modal.update_preview(40, 20);
        assert!(modal.cached_preview.is_some());
        assert_eq!(modal.last_preview_dims, Some((40, 20)));

        // Resizing window dynamically re-evaluates preview
        modal.update_preview(60, 30);
        assert!(modal.cached_preview.is_some());
        assert_eq!(modal.last_preview_dims, Some((60, 30)));

        let _ = std::fs::remove_file(&temp_path);
    }
}
