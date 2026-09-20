use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};
use crate::domain::favorite::FavoriteManga;
use crate::i18n::AppLanguage;

pub struct FavoritesListComponent;

impl FavoritesListComponent {
    pub fn render(
        frame: &mut Frame,
        area: Rect,
        favorites: &[FavoriteManga],
        state: &mut ListState,
        is_focused: bool,
        lang: AppLanguage,
    ) {
        let border_style = if is_focused {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = match lang {
            AppLanguage::English => format!(" 󰓎 Favorite Mangas ({}) ", favorites.len()),
            AppLanguage::Portuguese => format!(" 󰓎 Mangás Favoritos ({}) ", favorites.len()),
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(border_style);

        if favorites.is_empty() {
            let empty_text = match lang {
                AppLanguage::English => vec![
                    Line::from(""),
                    Line::from(Span::styled("󰓍 No favorite mangas saved yet", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled("Press [/] or [Enter] above to search for manga.", Style::default().fg(Color::White))),
                    Line::from(Span::styled("In the search results, press [f] to bookmark any series!", Style::default().fg(Color::DarkGray))),
                ],
                AppLanguage::Portuguese => vec![
                    Line::from(""),
                    Line::from(Span::styled("󰓍 Nenhum mangá favorito salvo ainda", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled("Pressione [/] ou [Enter] acima para pesquisar mangás.", Style::default().fg(Color::White))),
                    Line::from(Span::styled("Na lista de resultados, aperte [f] para favoritar uma obra!", Style::default().fg(Color::DarkGray))),
                ],
            };
            let paragraph = Paragraph::new(empty_text)
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(paragraph, area);
            return;
        }

        let items: Vec<ListItem> = favorites
            .iter()
            .enumerate()
            .map(|(idx, fav)| {
                let prefix = format!("{:2}. ", idx + 1);
                let mut spans = vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled("★ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::styled(&fav.title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" [{}]", fav.provider), Style::default().fg(Color::Magenta)),
                ];

                if let Some(ref last_ch) = fav.last_downloaded_chapter {
                    let label = match lang {
                        AppLanguage::English => format!("  󰄬 Last: {}", last_ch),
                        AppLanguage::Portuguese => format!("  󰄬 Último: {}", last_ch),
                    };
                    spans.push(Span::styled(label, Style::default().fg(Color::Cyan)));
                } else {
                    let label = match lang {
                        AppLanguage::English => "  󰋩 Not downloaded yet".to_string(),
                        AppLanguage::Portuguese => "  󰋩 Nenhum capítulo baixado".to_string(),
                    };
                    spans.push(Span::styled(label, Style::default().fg(Color::DarkGray)));
                }

                ListItem::new(Line::from(spans))
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
