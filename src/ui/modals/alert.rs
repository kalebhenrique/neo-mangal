use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};
use crate::i18n::AppLanguage;

pub struct AlertModal {
    pub title: String,
    pub message: String,
    pub is_error: bool,
}

impl AlertModal {
    pub fn new(title: &str, message: &str, is_error: bool) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            is_error,
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let lines_count = self.message.lines().count();
        let h = ((lines_count as u16) + 6).clamp(8, 14).min(area.height.saturating_sub(2));
        let w = 62.min(area.width.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        frame.render_widget(Clear, modal_area);

        let icon = if self.is_error { "󰅚" } else { "󰄬" };
        let border_color = if self.is_error { Color::Red } else { Color::Green };

        let trimmed_title = self.title.trim();
        let title_text = if trimmed_title.chars().next().map_or(false, |c| !c.is_ascii()) {
            format!(" {} ", trimmed_title)
        } else {
            format!(" {} {} ", icon, trimmed_title)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title_text)
            .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let show_spacer = inner.height >= 5;
        let constraints = if show_spacer {
            vec![
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ]
        } else {
            vec![
                Constraint::Min(1),
                Constraint::Length(1),
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(inner);

        let msg_chunk = if show_spacer { chunks[1] } else { chunks[0] };
        let footer_chunk = if show_spacer { chunks[2] } else { chunks[1] };

        // Message content with contextual styling
        let mut message_lines = Vec::new();
        for line in self.message.lines() {
            if !self.is_error && (line.starts_with('/') || line.contains(":\\")) {
                message_lines.push(Line::from(vec![
                    Span::styled(line, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            } else if !self.is_error && line.starts_with("AniList:") {
                let rest = line.trim_start_matches("AniList:").trim();
                message_lines.push(Line::from(vec![
                    Span::styled("AniList: ", Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)),
                    Span::styled(rest, Style::default().fg(Color::White)),
                ]));
            } else {
                let color = if self.is_error { Color::LightRed } else { Color::White };
                message_lines.push(Line::from(vec![
                    Span::styled(line, Style::default().fg(color)),
                ]));
            }
        }

        let message_p = Paragraph::new(message_lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        frame.render_widget(message_p, msg_chunk);

        // Standardized footer matching other modals
        let footer_spans: Vec<Span> = match lang {
            AppLanguage::English => vec![
                Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("OK      ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Close", Style::default().fg(Color::White)),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("OK      ", Style::default().fg(Color::White)),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Fechar", Style::default().fg(Color::White)),
            ],
        };

        frame.render_widget(
            Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center),
            footer_chunk,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_modal_new() {
        let alert = AlertModal::new("Sucesso!", "Capítulo 1 marcado como lido.", false);
        assert_eq!(alert.title, "Sucesso!");
        assert_eq!(alert.message, "Capítulo 1 marcado como lido.");
        assert!(!alert.is_error);

        let err_alert = AlertModal::new("Erro", "Falha de conexão", true);
        assert!(err_alert.is_error);
    }
}
