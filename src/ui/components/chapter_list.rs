use crate::domain::manga::Chapter;
use crate::i18n::{AppLanguage, I18n};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use std::collections::HashSet;

pub struct ChapterListComponent;

impl ChapterListComponent {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        manga_title: &str,
        page_chapters: &[Chapter],
        current_page: usize,
        total_pages: usize,
        total_matching: usize,
        selected_ids: &HashSet<String>,
        filter_query: &str,
        is_filtering: bool,
        state: &mut ListState,
        lang: AppLanguage,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Filter & Pagination Header
                Constraint::Min(6),    // Chapters list for this page
            ])
            .split(area);

        // 1. Filter input and pagination indicator
        let filter_border_style = if is_filtering {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let filter_block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::filter_chapters_title(lang, manga_title))
            .border_style(filter_border_style);

        let cursor = if is_filtering { "█" } else { "" };
        let filter_line = Line::from(vec![
            Span::styled(I18n::filter_label(lang), Style::default().fg(Color::Cyan)),
            Span::styled(
                filter_query,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(cursor, Style::default().fg(Color::Yellow)),
            Span::raw("   |   "),
            Span::styled(
                I18n::pagination_info(lang, current_page + 1, total_pages.max(1), total_matching),
                Style::default().fg(Color::LightGreen),
            ),
            Span::styled(
                I18n::pagination_hint(lang),
                Style::default().fg(Color::DarkGray),
            ),
        ]);
        frame.render_widget(Paragraph::new(filter_line).block(filter_block), chunks[0]);

        // 2. Chapters List
        let list_border_style = if !is_filtering {
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let list_block = Block::default()
            .borders(Borders::ALL)
            .title(I18n::chapters_box_title(lang, page_chapters.len()))
            .border_style(list_border_style);

        let items: Vec<ListItem> = page_chapters
            .iter()
            .map(|ch| {
                let is_checked = selected_ids.contains(&ch.id);
                let check_mark = if is_checked {
                    Span::styled(
                        "󰄲 ",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled("󰄱 ", Style::default().fg(Color::DarkGray))
                };

                let content = Line::from(vec![
                    check_mark,
                    Span::styled(&ch.title, Style::default().fg(Color::White)),
                ]);
                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(list_block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(50, 45, 80))
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(list, chunks[1], state);
    }
}
