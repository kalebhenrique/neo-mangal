use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;

pub struct ConfirmMarkReadModal {
    pub manga_title: String,
    pub chapter_number: f32,
    pub chapter_title: String,
    pub token: String,
}

impl ConfirmMarkReadModal {
    pub fn new(manga_title: &str, chapter_number: f32, chapter_title: &str, token: &str) -> Self {
        Self {
            manga_title: manga_title.to_string(),
            chapter_number,
            chapter_title: chapter_title.to_string(),
            token: token.to_string(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let w = 66.min(area.width.saturating_sub(2));
        let h = 11.min(area.height.saturating_sub(2));
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect { x, y, width: w, height: h };

        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰚥 Sync Reading on AniList ",
            AppLanguage::Portuguese => " 󰚥 Marcar Leitura no AniList ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(4),    // Info & confirmation text
                Constraint::Length(1), // Footer keys
            ])
            .split(inner);

        let ch_num_display = if self.chapter_number.fract() == 0.0 {
            format!("{}", self.chapter_number as i32)
        } else {
            format!("{}", self.chapter_number)
        };

        let mut lines = Vec::new();

        // 1. Manga line
        lines.push(Line::from(vec![
            Span::styled(
                match lang {
                    AppLanguage::English => "  Manga:    ",
                    AppLanguage::Portuguese => "  Mangá:    ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(&self.manga_title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));

        // 2. Chapter line
        lines.push(Line::from(vec![
            Span::styled(
                match lang {
                    AppLanguage::English => "  Chapter:  ",
                    AppLanguage::Portuguese => "  Capítulo: ",
                },
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                format!("Ch. {} ", ch_num_display),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({})", self.chapter_title),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        lines.push(Line::raw(""));

        // 3. Question & consequence
        lines.push(Line::from(vec![
            Span::styled(
                match lang {
                    AppLanguage::English => "  Update your AniList library reading progress to this chapter?",
                    AppLanguage::Portuguese => "  Deseja marcar este capítulo como lido no seu AniList?",
                },
                Style::default().fg(Color::White),
            ),
        ]));

        lines.push(Line::from(vec![
            Span::styled(
                match lang {
                    AppLanguage::English => format!("  Progress will be set to Chapter {}.", ch_num_display),
                    AppLanguage::Portuguese => format!("  O progresso será atualizado para o Capítulo {}.", ch_num_display),
                },
                Style::default().fg(Color::Green),
            ),
        ]));

        frame.render_widget(Paragraph::new(lines), chunks[0]);

        // Footer buttons
        let footer_spans: Vec<Span> = match lang {
            AppLanguage::English => vec![
                Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Confirm      "),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Cancel"),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled("[Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Confirmar      "),
                Span::styled("[Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Cancelar"),
            ],
        };

        frame.render_widget(Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center), chunks[1]);
    }
}
