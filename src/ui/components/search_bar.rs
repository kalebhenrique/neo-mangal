use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::i18n::{AppLanguage, I18n};

pub struct SearchBarComponent;

impl SearchBarComponent {
    pub fn render(frame: &mut Frame, area: Rect, input: &str, is_focused: bool, lang: AppLanguage) {
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::search_title(lang))
            .border_style(border_style);

        let display_text = if input.is_empty() && !is_focused {
            Line::from(vec![
                Span::styled(I18n::search_placeholder(lang), Style::default().fg(Color::DarkGray))
            ])
        } else {
            let cursor = if is_focused { "█" } else { "" };
            Line::from(vec![
                Span::styled("> ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(input, Style::default().fg(Color::White)),
                Span::styled(cursor, Style::default().fg(Color::Yellow)),
            ])
        };

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, area);
    }
}
