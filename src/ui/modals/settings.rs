use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::Path;
use crate::config::Config;
use crate::i18n::{AppLanguage, I18n};

#[derive(Clone, Debug)]
struct AutocompleteState {
    base_dir: String,
    matches: Vec<String>,
    current_index: usize,
}

pub struct SettingsModal {
    pub input_path: String,
    pub language: String,
    pub profile: String,
    pub format: String,
    autocomplete_state: Option<AutocompleteState>,
}

impl SettingsModal {
    pub fn new(config: &Config) -> Self {
        Self {
            input_path: config.download_dir.to_string_lossy().to_string(),
            language: config.language.clone(),
            profile: config.kcc_profile.clone(),
            format: config.kcc_format.clone(),
            autocomplete_state: None,
        }
    }

    pub fn current_lang(&self) -> AppLanguage {
        AppLanguage::from_code(&self.language)
    }

    pub fn handle_char(&mut self, c: char) {
        self.autocomplete_state = None;
        self.input_path.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.autocomplete_state = None;
        self.input_path.pop();
    }

    pub fn autocomplete_path(&mut self) {
        // If already cycling through autocomplete matches, advance to next match
        if let Some(ref mut state) = self.autocomplete_state {
            if !state.matches.is_empty() {
                state.current_index = (state.current_index + 1) % state.matches.len();
                let choice = &state.matches[state.current_index];
                self.input_path = format!("{}{}/", state.base_dir, choice);
                return;
            }
        }

        let raw = self.input_path.trim().to_string();
        if raw.is_empty() {
            if let Some(home) = dirs::home_dir() {
                self.input_path = format!("{}/", home.display());
            } else {
                self.input_path = "./".to_string();
            }
            return;
        }

        // Split into base_dir (directory to search) and prefix (item name to complete)
        let (base_dir, prefix) = if let Some(idx) = raw.rfind('/') {
            (raw[..=idx].to_string(), raw[idx + 1..].to_string())
        } else if let Some(idx) = raw.rfind('\\') {
            (raw[..=idx].to_string(), raw[idx + 1..].to_string())
        } else {
            ("".to_string(), raw)
        };

        // Resolve base_dir to filesystem path
        let resolved_dir: std::path::PathBuf = if base_dir.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&base_dir[2..])
            } else {
                std::path::PathBuf::from(&base_dir)
            }
        } else if base_dir == "~" {
            dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."))
        } else if base_dir.is_empty() {
            std::path::PathBuf::from(".")
        } else {
            std::path::PathBuf::from(&base_dir)
        };

        if !resolved_dir.is_dir() {
            return;
        }

        // Read entries in resolved_dir
        let mut matches = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&resolved_dir) {
            let prefix_lower = prefix.to_lowercase();
            for entry in entries.flatten() {
                let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                if is_dir {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with('.') && !prefix.starts_with('.') {
                        continue;
                    }
                    if name.to_lowercase().starts_with(&prefix_lower) {
                        matches.push(name);
                    }
                }
            }
        }

        if matches.is_empty() {
            return;
        }

        matches.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));

        let common = find_common_prefix(&matches);

        if matches.len() == 1 {
            self.input_path = format!("{}{}/", base_dir, matches[0]);
            self.autocomplete_state = Some(AutocompleteState {
                base_dir,
                matches,
                current_index: 0,
            });
        } else if common.len() > prefix.len() {
            let is_exact = matches.iter().any(|m| m == &common);
            if is_exact {
                self.input_path = format!("{}{}/", base_dir, common);
            } else {
                self.input_path = format!("{}{}", base_dir, common);
            }
            self.autocomplete_state = Some(AutocompleteState {
                base_dir,
                matches,
                current_index: 0,
            });
        } else {
            self.input_path = format!("{}{}/", base_dir, matches[0]);
            self.autocomplete_state = Some(AutocompleteState {
                base_dir,
                matches,
                current_index: 0,
            });
        }
    }

    pub fn cycle_profile(&mut self) {
        let profiles = ["KPW5", "KV", "KO", "KS", "K11", "KPW"];
        if let Some(pos) = profiles.iter().position(|&p| p == self.profile) {
            let next_idx = (pos + 1) % profiles.len();
            self.profile = profiles[next_idx].to_string();
        } else {
            self.profile = "KPW5".to_string();
        }
    }

    pub fn cycle_format(&mut self) {
        let formats = ["AZW3", "CBZ", "EPUB", "MOBI"];
        if let Some(pos) = formats.iter().position(|&f| f.eq_ignore_ascii_case(&self.format)) {
            let next_idx = (pos + 1) % formats.len();
            self.format = formats[next_idx].to_string();
        } else {
            self.format = "AZW3".to_string();
        }
    }

    pub fn cycle_language(&mut self) {
        match self.current_lang() {
            AppLanguage::English => {
                self.language = "pt-br".to_string();
            }
            AppLanguage::Portuguese => {
                self.language = "en".to_string();
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lang = self.current_lang();
        let modal_area = centered_rect(75, 65, area);
        frame.render_widget(Clear, modal_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::settings_title(lang))
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Output dir input
                Constraint::Length(2), // Validation message
                Constraint::Length(3), // Language selector
                Constraint::Length(4), // KCC Profile & description
                Constraint::Min(2),    // Shortcut instructions
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // 1. Output directory input
        let path_block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::download_dir_title(lang))
            .border_style(Style::default().fg(Color::Cyan));
        let path_text = Line::from(vec![
            Span::styled(&self.input_path, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        frame.render_widget(Paragraph::new(path_text).block(path_block), chunks[0]);

        // 2. Directory validation hint
        let path_ref = Path::new(&self.input_path);
        let validation_text = if path_ref.is_dir() {
            Line::from(vec![
                Span::styled(I18n::dir_valid(lang), Style::default().fg(Color::Green)),
            ])
        } else if !self.input_path.trim().is_empty() {
            Line::from(vec![
                Span::styled(I18n::dir_will_create(lang), Style::default().fg(Color::Yellow)),
            ])
        } else {
            Line::from(vec![
                Span::styled(I18n::dir_empty_err(lang), Style::default().fg(Color::Red)),
            ])
        };
        frame.render_widget(Paragraph::new(validation_text), chunks[1]);

        // 3. Language selector
        let lang_block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::lang_title(lang))
            .border_style(Style::default().fg(Color::DarkGray));
        let lang_line = Line::from(vec![
            Span::raw(I18n::lang_label(lang)),
            Span::styled(format!("[ {} ]", self.language), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" - {}", lang.display_name()), Style::default().fg(Color::White)),
        ]);
        frame.render_widget(Paragraph::new(lang_line).block(lang_block), chunks[2]);

        // 4. KCC Profile & Format
        let profile_desc = match (self.profile.as_str(), lang) {
            ("KPW5", AppLanguage::English) => "Kindle Paperwhite (11th / 12th Gen - 300 ppi)",
            ("KPW5", AppLanguage::Portuguese) => "Kindle Paperwhite (11ª / 12ª Geração - 300 ppi)",
            ("KV", AppLanguage::English) => "Kindle Paperwhite (3rd / 4th Gen) / Voyage",
            ("KV", AppLanguage::Portuguese) => "Kindle Paperwhite (3ª / 4ª Geração) / Voyage",
            ("KO", AppLanguage::English) => "Kindle Oasis (2nd / 3rd Gen - 300 ppi)",
            ("KO", AppLanguage::Portuguese) => "Kindle Oasis (2ª / 3ª Geração - 300 ppi)",
            ("KS", AppLanguage::English) => "Kindle Scribe (10.2\" - 300 ppi)",
            ("KS", AppLanguage::Portuguese) => "Kindle Scribe (10.2\" - 300 ppi)",
            ("K11", AppLanguage::English) => "Kindle Basic (11th Gen)",
            ("K11", AppLanguage::Portuguese) => "Kindle Básico (11ª Geração)",
            ("KPW", AppLanguage::English) => "Kindle Paperwhite (Legacy 1st / 2nd Gen)",
            ("KPW", AppLanguage::Portuguese) => "Kindle Paperwhite (1ª / 2ª Geração antiga)",
            _ => "Kindle Device",
        };

        let profile_block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::kcc_profile_title(lang))
            .border_style(Style::default().fg(Color::DarkGray));
        let profile_line1 = Line::from(vec![
            Span::raw(I18n::model_label(lang)),
            Span::styled(format!("[ {} ]", self.profile), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw(I18n::format_label(lang)),
            Span::styled(format!("[ {} ]", self.format), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]);
        let profile_line2 = Line::from(vec![
            Span::styled(format!("↳ {}", profile_desc), Style::default().fg(Color::LightGreen)),
        ]);
        frame.render_widget(
            Paragraph::new(vec![profile_line1, profile_line2]).block(profile_block),
            chunks[3],
        );

        // 5. Instructions
        let instructions_data = I18n::settings_instructions(lang);
        let spans: Vec<Span> = instructions_data
            .into_iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(desc),
                ]
            })
            .collect();
        frame.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), chunks[4]);
    }
}

fn find_common_prefix(strs: &[String]) -> String {
    if strs.is_empty() {
        return String::new();
    }
    let mut prefix = strs[0].clone();
    for s in &strs[1..] {
        while !s.to_lowercase().starts_with(&prefix.to_lowercase()) {
            if prefix.is_empty() {
                return String::new();
            }
            prefix.pop();
        }
    }
    prefix
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cycle_format() {
        let config = Config::default();
        let mut modal = SettingsModal::new(&config);
        assert_eq!(modal.format, "AZW3");

        modal.cycle_format();
        assert_eq!(modal.format, "CBZ");

        modal.cycle_format();
        assert_eq!(modal.format, "EPUB");

        modal.cycle_format();
        assert_eq!(modal.format, "MOBI");

        modal.cycle_format();
        assert_eq!(modal.format, "AZW3");
    }

    #[test]
    fn test_find_common_prefix() {
        let items = vec!["Downloads".to_string(), "Documents".to_string()];
        assert_eq!(find_common_prefix(&items), "Do");

        let items2 = vec!["Manga".to_string(), "Mangadex".to_string(), "Mangal".to_string()];
        assert_eq!(find_common_prefix(&items2), "Manga");
    }

    #[test]
    fn test_autocomplete_path() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_autocomplete_test");
        let _ = std::fs::create_dir_all(temp_dir.join("folder_alpha"));
        let _ = std::fs::create_dir_all(temp_dir.join("folder_beta"));

        let config = Config::default();
        let mut modal = SettingsModal::new(&config);

        // Test with prefix
        modal.input_path = format!("{}/folder_a", temp_dir.display());
        modal.autocomplete_path();
        assert_eq!(modal.input_path, format!("{}/folder_alpha/", temp_dir.display()));

        // Clean up
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}

