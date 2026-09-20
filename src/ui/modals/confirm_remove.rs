use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::domain::manga::Manga;
use crate::i18n::AppLanguage;

#[derive(Debug, Clone)]
pub struct ConfirmRemoveModal {
    pub manga: Manga,
}

impl ConfirmRemoveModal {
    pub fn new(manga: Manga) -> Self {
        Self { manga }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let w = (area.width * 55 / 100).max(46).min(area.width);
        let h = 9.max(area.height * 28 / 100).min(area.height);
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect {
            x,
            y,
            width: w,
            height: h,
        };

        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰆴 Remove from Favorites ",
            AppLanguage::Portuguese => " 󰆴 Remover dos Favoritos ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Question
                Constraint::Length(1), // Manga title
                Constraint::Min(1),    // Provider / spacer
                Constraint::Length(1), // Footer key hints
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        let question = match lang {
            AppLanguage::English => "Are you sure you want to remove this manga from favorites?",
            AppLanguage::Portuguese => "Tem certeza que deseja remover este mangá dos favoritos?",
        };

        let q_p = Paragraph::new(Line::from(vec![
            Span::styled(question, Style::default().fg(Color::White)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(q_p, chunks[0]);

        let title_p = Paragraph::new(Line::from(vec![
            Span::styled(
                format!("\"{}\"", self.manga.title),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(title_p, chunks[1]);

        let provider_text = match lang {
            AppLanguage::English => format!("Source: {}", self.manga.provider),
            AppLanguage::Portuguese => format!("Fonte: {}", self.manga.provider),
        };
        let prov_p = Paragraph::new(Line::from(vec![
            Span::styled(provider_text, Style::default().fg(Color::DarkGray)),
        ]))
        .alignment(Alignment::Center);
        frame.render_widget(prov_p, chunks[2]);

        let footer_line = match lang {
            AppLanguage::English => Line::from(vec![
                Span::styled("[Enter/y] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Yes   ", Style::default().fg(Color::White)),
                Span::styled("[Esc/n] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("No", Style::default().fg(Color::White)),
            ]),
            AppLanguage::Portuguese => Line::from(vec![
                Span::styled("[Enter/s] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled("Sim   ", Style::default().fg(Color::White)),
                Span::styled("[Esc/n] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::styled("Não", Style::default().fg(Color::White)),
            ]),
        };
        let footer_p = Paragraph::new(footer_line).alignment(Alignment::Center);
        frame.render_widget(footer_p, chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_remove_modal_creation() {
        let manga = Manga {
            id: "m1".into(),
            title: "Naruto".into(),
            url: "https://example.com/naruto".into(),
            cover_url: None,
            provider: "MangaDex".into(),
        };
        let modal = ConfirmRemoveModal::new(manga.clone());
        assert_eq!(modal.manga.title, "Naruto");
        assert_eq!(modal.manga.provider, "MangaDex");
    }
}
