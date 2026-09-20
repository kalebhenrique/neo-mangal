use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::ui::modals::settings::centered_rect;

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

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let modal_area = centered_rect(60, 30, area);
        frame.render_widget(Clear, modal_area);

        let border_color = if self.is_error { Color::Red } else { Color::Green };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.title))
            .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Min(2),
                Constraint::Length(1),
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        let message_p = Paragraph::new(self.message.clone())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(message_p, chunks[0]);

        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[Enter] / [Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw("OK"),
        ])).alignment(Alignment::Center);
        frame.render_widget(footer, chunks[1]);
    }
}
