use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::i18n::{AppLanguage, I18n};

pub struct HeaderComponent;

impl HeaderComponent {
    pub fn render(frame: &mut Frame, area: Rect, provider: &str, is_busy: bool, lang: AppLanguage) {
        let status_span = if is_busy {
            Span::styled(
                I18n::status_busy(lang),
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(
                I18n::status_ready(lang),
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
            )
        };

        let title_line = Line::from(vec![
            Span::styled(" NEO-MANGAL ", Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" v{} ", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::DarkGray)),
            Span::raw(" |  Source: "),
            Span::styled(provider, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            status_span,
        ]);

        let paragraph = Paragraph::new(title_line)
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::BOTTOM).style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(paragraph, area);
    }
}
