use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::Path;
use crate::i18n::{AppLanguage, I18n};

#[derive(Clone, Debug)]
struct AutocompleteState {
    base_dir: String,
    matches: Vec<String>,
    current_index: usize,
}

pub struct EditPathModal {
    pub input_path: String,
    autocomplete_state: Option<AutocompleteState>,
}

impl EditPathModal {
    pub fn new(current_path: &str) -> Self {
        Self {
            input_path: current_path.to_string(),
            autocomplete_state: None,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }
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

        // Split into base_dir and prefix
        let (base_dir, prefix) = if let Some(idx) = raw.rfind('/') {
            (raw[..=idx].to_string(), raw[idx + 1..].to_string())
        } else if let Some(idx) = raw.rfind('\\') {
            (raw[..=idx].to_string(), raw[idx + 1..].to_string())
        } else {
            ("".to_string(), raw)
        };

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

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let w = 72.min(area.width.saturating_sub(2));
        let h = 9.min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect { x, y, width: w, height: h };

        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰉋 Output / Download Directory ",
            AppLanguage::Portuguese => " 󰉋 Diretório de Download / Saída ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Length(3), // Input box
                Constraint::Length(1), // Validation hint
                Constraint::Min(1),    // Footer
            ])
            .split(inner_area);

        // 1. Input Box
        let mut display_path = self.input_path.clone();
        if display_path.len() > 55 {
            display_path = format!("...{}", &display_path[display_path.len() - 52..]);
        }
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(match lang {
                AppLanguage::English => " Directory Path ",
                AppLanguage::Portuguese => " Caminho da Pasta ",
            })
            .border_style(Style::default().fg(Color::Yellow));

        let line = Line::from(vec![
            Span::styled(display_path, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        frame.render_widget(Paragraph::new(line).block(input_block), chunks[0]);

        // 2. Validation Hint
        let path_ref = Path::new(&self.input_path);
        let status_span = if path_ref.is_dir() {
            Span::styled(I18n::dir_valid(lang), Style::default().fg(Color::Green))
        } else if !self.input_path.trim().is_empty() {
            Span::styled(I18n::dir_will_create(lang), Style::default().fg(Color::Yellow))
        } else {
            Span::styled(I18n::dir_empty_err(lang), Style::default().fg(Color::Red))
        };
        frame.render_widget(Paragraph::new(Line::from(vec![Span::raw(" "), status_span])), chunks[1]);

        // 3. Footer
        let footer_spans: Vec<Span> = match lang {
            AppLanguage::English => vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Autocomplete   "),
                Span::styled("[Enter] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Confirm   "),
                Span::styled("[Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Cancel"),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled("[Tab] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Autocompletar   "),
                Span::styled("[Enter] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Confirmar   "),
                Span::styled("[Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Cancelar"),
            ],
        };
        frame.render_widget(Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center), chunks[2]);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_path_modal_typing_and_autocomplete() {
        let temp_dir = std::env::temp_dir().join("neo_mangal_edit_path_test");
        let _ = std::fs::create_dir_all(temp_dir.join("folder_alpha"));
        let _ = std::fs::create_dir_all(temp_dir.join("folder_beta"));

        let mut modal = EditPathModal::new(&format!("{}/folder_a", temp_dir.display()));
        modal.handle_char('l');
        modal.handle_backspace();
        modal.autocomplete_path();
        assert_eq!(modal.input_path, format!("{}/folder_alpha/", temp_dir.display()));

        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
