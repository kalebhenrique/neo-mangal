use std::collections::HashSet;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;
use crate::ui::modals::settings::centered_rect;

#[derive(Debug, Clone)]
pub struct SourceSelectModal {
    pub sources: Vec<String>,
    pub selected_idx: usize,
    pub checked_sources: HashSet<String>,
}

impl SourceSelectModal {
    pub fn new(sources: Vec<String>, preselected: &[String]) -> Self {
        let mut checked_sources = HashSet::new();
        for s in preselected {
            if sources.contains(s) {
                checked_sources.insert(s.clone());
            }
        }
        // If nothing was preselected, mark all available sources by default
        if checked_sources.is_empty() {
            for s in &sources {
                checked_sources.insert(s.clone());
            }
        }

        Self {
            sources,
            selected_idx: 0,
            checked_sources,
        }
    }

    pub fn toggle_check(&mut self) {
        if let Some(src) = self.sources.get(self.selected_idx) {
            if self.checked_sources.contains(src) {
                self.checked_sources.remove(src);
            } else {
                self.checked_sources.insert(src.clone());
            }
        }
    }

    pub fn toggle_all(&mut self) {
        if self.checked_sources.len() == self.sources.len() {
            self.checked_sources.clear();
        } else {
            for s in &self.sources {
                self.checked_sources.insert(s.clone());
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.sources.is_empty() {
            return;
        }
        if self.selected_idx == 0 {
            self.selected_idx = self.sources.len() - 1;
        } else {
            self.selected_idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.sources.is_empty() {
            return;
        }
        if self.selected_idx + 1 >= self.sources.len() {
            self.selected_idx = 0;
        } else {
            self.selected_idx += 1;
        }
    }

    pub fn get_selected_sources(&self) -> Vec<String> {
        let mut list: Vec<String> = self.sources
            .iter()
            .filter(|s| self.checked_sources.contains(*s))
            .cloned()
            .collect();

        // Fallback: if user unchecked everything, pick the one under the cursor
        if list.is_empty() {
            if let Some(src) = self.sources.get(self.selected_idx) {
                list.push(src.clone());
            }
        }
        list
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let modal_area = centered_rect(56, 60, area);
        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰋩 Select Manga Sources ",
            AppLanguage::Portuguese => " 󰋩 Selecionar Fontes de Mangá ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(2), // Subtitle / hint
                Constraint::Min(4),    // List of scrapers
                Constraint::Length(2), // Actions footer
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // 1. Subtitle Hint
        let count_checked = self.checked_sources.len();
        let total = self.sources.len();
        let hint_text = match lang {
            AppLanguage::English => format!(
                "Press [Space] to select/deselect ({}/{} sources active):",
                count_checked, total
            ),
            AppLanguage::Portuguese => format!(
                "Pressione [Espaço] para marcar/desmarcar ({}/{} fontes ativas):",
                count_checked, total
            ),
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("󰄬 ", Style::default().fg(Color::Yellow)),
                Span::styled(hint_text, Style::default().fg(Color::White)),
            ])),
            chunks[0],
        );

        // 2. Sources List with scroll calculation
        let list_area = chunks[1];
        let height = list_area.height as usize;

        let (start_idx, end_idx) = if total <= height {
            (0, total)
        } else {
            let offset = if self.selected_idx >= height {
                self.selected_idx - height + 1
            } else {
                0
            };
            (offset, (offset + height).min(total))
        };

        let mut lines = Vec::new();
        for (i, source) in self.sources.iter().enumerate().skip(start_idx).take(end_idx - start_idx) {
            let is_cursor = i == self.selected_idx;
            let is_checked = self.checked_sources.contains(source);

            let mut spans = Vec::new();

            // Cursor prefix
            if is_cursor {
                spans.push(Span::styled(" ▶ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::raw("   "));
            }

            // Checkbox icon
            if is_checked {
                spans.push(Span::styled("󰄲 ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)));
            } else {
                spans.push(Span::styled("󰄱 ", Style::default().fg(Color::DarkGray)));
            }

            // Source name
            let name_style = if is_cursor {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else if is_checked {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            spans.push(Span::styled(format!("{:<18}", source), name_style));

            // Status label
            if is_checked {
                let badge = match lang {
                    AppLanguage::English => " [Active]",
                    AppLanguage::Portuguese => " [Ativa]",
                };
                spans.push(Span::styled(
                    badge,
                    Style::default().fg(Color::Green),
                ));
            }

            lines.push(Line::from(spans));
        }

        frame.render_widget(Paragraph::new(lines), chunks[1]);

        // 3. Actions footer
        let action_spans = match lang {
            AppLanguage::English => vec![
                Span::styled(" [Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Toggle  "),
                Span::styled(" [a] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("All  "),
                Span::styled(" [Enter/Esc] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Save & Close  "),
                Span::styled(" [q] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Quit"),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled(" [Espaço] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Marcar  "),
                Span::styled(" [a] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Todos  "),
                Span::styled(" [Enter/Esc] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Salvar e Fechar  "),
                Span::styled(" [q] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Sair"),
            ],
        };

        frame.render_widget(
            Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center),
            chunks[2],
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_source_selection() {
        let sources = vec![
            "MangaDex".to_string(),
            "WeebCentral".to_string(),
        ];
        let mut modal = SourceSelectModal::new(sources.clone(), &["WeebCentral".to_string()]);
        assert_eq!(modal.checked_sources.len(), 1);
        assert!(modal.checked_sources.contains("WeebCentral"));

        // Toggle first (MangaDex) with space
        modal.selected_idx = 0;
        modal.toggle_check();
        assert_eq!(modal.checked_sources.len(), 2);
        assert!(modal.checked_sources.contains("MangaDex"));

        // Toggle second (WeebCentral) off
        modal.selected_idx = 1;
        modal.toggle_check();
        assert_eq!(modal.checked_sources.len(), 1);
        assert!(!modal.checked_sources.contains("WeebCentral"));

        // Toggle all
        modal.toggle_all();
        assert_eq!(modal.checked_sources.len(), 2);
    }
}
