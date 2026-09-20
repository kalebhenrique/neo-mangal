use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;

pub struct StatusBarComponent;

impl StatusBarComponent {
    pub fn render(frame: &mut Frame, area: Rect, current_view: &str, output_path: &str, lang: AppLanguage) {
        let key_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::White);

        let mut spans = match (current_view, lang) {
            // Search view
            ("search", AppLanguage::English) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Search  ", desc_style),
                Span::styled(" [Esc] ", key_style),
                Span::styled("Unfocus  ", desc_style),
                Span::styled(" [Ctrl+P] ", key_style),
                Span::styled("Source  ", desc_style),
                Span::styled(" [F2] ", key_style),
                Span::styled("Settings", desc_style),
            ],
            ("search", AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Buscar  ", desc_style),
                Span::styled(" [Esc] ", key_style),
                Span::styled("Desfocar  ", desc_style),
                Span::styled(" [Ctrl+P] ", key_style),
                Span::styled("Fonte  ", desc_style),
                Span::styled(" [F2] ", key_style),
                Span::styled("Configurações", desc_style),
            ],

            // Search unfocused view
            ("search_unfocused", AppLanguage::English) => vec![
                Span::styled(" [/]/[Enter] ", key_style),
                Span::styled("Focus Search  ", desc_style),
                Span::styled(" [p] ", key_style),
                Span::styled("Source  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Settings  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Quit", desc_style),
            ],
            ("search_unfocused", AppLanguage::Portuguese) => vec![
                Span::styled(" [/]/[Enter] ", key_style),
                Span::styled("Focar Busca  ", desc_style),
                Span::styled(" [p] ", key_style),
                Span::styled("Fonte  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Configurações  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Sair", desc_style),
            ],

            // Manga results view
            ("manga_list", AppLanguage::English) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("View Chapters  ", desc_style),
                Span::styled(" [/] ", key_style),
                Span::styled("New Search  ", desc_style),
                Span::styled(" [p] ", key_style),
                Span::styled("Source  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Settings  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Quit", desc_style),
            ],
            ("manga_list", AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Ver Capítulos  ", desc_style),
                Span::styled(" [/] ", key_style),
                Span::styled("Nova Busca  ", desc_style),
                Span::styled(" [p] ", key_style),
                Span::styled("Fonte  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Configurações  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Sair", desc_style),
            ],

            // Chapter list view
            ("chapter_list", AppLanguage::English) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Process/Download  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" [Space] ", key_style),
                Span::styled("Check  ", desc_style),
                Span::styled(" [a] ", key_style),
                Span::styled("All  ", desc_style),
                Span::styled(" [/] ", key_style),
                Span::styled("Filter  ", desc_style),
                Span::styled(" [←/→] ", key_style),
                Span::styled("Pages  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Settings  ", desc_style),
                Span::styled(" [b/Esc] ", key_style),
                Span::styled("Back  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Quit", desc_style),
            ],
            ("chapter_list", AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter] ", key_style),
                Span::styled("Processar/Baixar  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::styled(" [Espaço] ", key_style),
                Span::styled("Marcar  ", desc_style),
                Span::styled(" [a] ", key_style),
                Span::styled("Todos  ", desc_style),
                Span::styled(" [/] ", key_style),
                Span::styled("Filtrar  ", desc_style),
                Span::styled(" [←/→] ", key_style),
                Span::styled("Páginas  ", desc_style),
                Span::styled(" [s] ", key_style),
                Span::styled("Config  ", desc_style),
                Span::styled(" [b/Esc] ", key_style),
                Span::styled("Voltar  ", desc_style),
                Span::styled(" [q] ", key_style),
                Span::styled("Sair", desc_style),
            ],

            // Chapter filter active view
            ("chapter_filter", AppLanguage::English) => vec![
                Span::styled(" [Esc]/[Enter] ", key_style),
                Span::styled("Done Filtering  ", desc_style),
                Span::styled(" [Backspace] ", key_style),
                Span::styled("Delete", desc_style),
            ],
            ("chapter_filter", AppLanguage::Portuguese) => vec![
                Span::styled(" [Esc]/[Enter] ", key_style),
                Span::styled("Concluir Filtro  ", desc_style),
                Span::styled(" [Backspace] ", key_style),
                Span::styled("Apagar", desc_style),
            ],

            _ => match lang {
                AppLanguage::English => vec![Span::styled(" [q] ", key_style), Span::styled("Quit", desc_style)],
                AppLanguage::Portuguese => vec![Span::styled(" [q] ", key_style), Span::styled("Sair", desc_style)],
            },
        };

        spans.push(Span::raw("  |  Out: "));
        spans.push(Span::styled(
            output_path,
            Style::default().fg(Color::Cyan),
        ));

        let paragraph = Paragraph::new(Line::from(spans))
            .block(Block::default().borders(Borders::TOP).style(Style::default().fg(Color::DarkGray)));

        frame.render_widget(paragraph, area);
    }
}
