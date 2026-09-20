use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};
use crate::domain::manga::Manga;
use crate::i18n::{AppLanguage, I18n};

pub struct MangaListComponent;

impl MangaListComponent {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        mangas: &[Manga],
        state: &mut ListState,
        is_focused: bool,
        lang: AppLanguage,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = I18n::series_found(lang, mangas.len());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        let items: Vec<ListItem> = mangas
            .iter()
            .enumerate()
            .map(|(idx, manga)| {
                let prefix = format!("{:2}. ", idx + 1);
                let content = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(&manga.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" [{}]", manga.provider), Style::default().fg(Color::Magenta)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(40, 50, 80))
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, area, state);
    }
}
