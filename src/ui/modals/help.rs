use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;

pub struct HelpModal {
    pub current_view: String,
}

impl HelpModal {
    pub fn new(current_view: &str) -> Self {
        Self {
            current_view: current_view.to_string(),
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let w = ((area.width as u32 * 85) / 100).clamp(64, 82) as u16;
        let w = w.min(area.width.saturating_sub(2));

        let h = ((area.height as u32 * 82) / 100).clamp(16, 23) as u16;
        let h = h.min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect { x, y, width: w, height: h };

        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰋖 Shortcuts & Help ",
            AppLanguage::Portuguese => " 󰋖 Atalhos & Ajuda ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));

        let inner = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Shortcuts list
                Constraint::Length(1), // Footer
            ])
            .split(inner);

        let key_col = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        let desc_col = Style::default().fg(Color::White);
        let section_col = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);

        let mut lines = Vec::new();

        // 1. Current screen shortcuts
        let section_title = match (self.current_view.as_str(), lang) {
            ("favorites", AppLanguage::English) => "[ Screen: Favorites ]",
            ("favorites", AppLanguage::Portuguese) => "[ Tela: Favoritos ]",
            ("chapter_list", AppLanguage::English) => "[ Screen: Chapters ]",
            ("chapter_list", AppLanguage::Portuguese) => "[ Tela: Capítulos ]",
            ("manga_list", AppLanguage::English) => "[ Screen: Search Results ]",
            ("manga_list", AppLanguage::Portuguese) => "[ Tela: Resultados da Busca ]",
            _ => match lang {
                AppLanguage::English => "[ Screen: Current View ]",
                AppLanguage::Portuguese => "[ Tela Atual ]",
            },
        };
        lines.push(Line::from(vec![Span::styled(section_title, section_col)]));

        let view_shortcuts: Vec<(&'static str, &'static str, &'static str)> = match self.current_view.as_str() {
            "chapter_list" => vec![
                ("[Enter]", "Download selected chapter(s)", "Baixar capítulo(s) selecionado(s)"),
                ("[m]", "Mark reading progress on AniList", "Marcar leitura no AniList (com confirmação)"),
                ("[Space]", "Toggle chapter selection checkbox", "Alternar seleção do capítulo"),
                ("[a]", "Select / Deselect all visible chapters", "Selecionar / Desmarcar todos os capítulos"),
                ("[/]", "Filter chapters by number / title", "Filtrar capítulos por número / título"),
                ("[← / →] / [h / l]", "Navigate chapter pages", "Navegar páginas de capítulos"),
                ("[b / Esc]", "Back to manga search / favorites", "Voltar para busca / favoritos"),
            ],
            "favorites" => vec![
                ("[Enter]", "Open chapters for selected manga", "Abrir capítulos do mangá selecionado"),
                ("[/]", "Focus search input", "Focar na barra de busca"),
                ("[f]", "Remove manga from favorites", "Remover mangá dos favoritos (com confirmação)"),
                ("[p]", "Manage & select manga sources", "Gerenciar e alternar fontes de mangá"),
            ],
            "manga_list" => vec![
                ("[Enter]", "View chapters for selected manga", "Ver capítulos do mangá selecionado"),
                ("[f]", "Add / Remove manga from favorites", "Adicionar / Remover dos favoritos"),
                ("[/]", "Start a new search query", "Focar na barra para nova busca"),
                ("[b / Esc]", "Back to favorites list", "Voltar para lista de favoritos"),
            ],
            _ => vec![
                ("[Enter]", "Confirm / Select action", "Confirmar / Selecionar ação"),
                ("[/]", "Focus search", "Focar na busca"),
                ("[Esc]", "Back / Unfocus", "Voltar / Desfocar"),
            ],
        };

        for (k, en_desc, pt_desc) in view_shortcuts {
            let desc = match lang {
                AppLanguage::English => en_desc,
                AppLanguage::Portuguese => pt_desc,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<20}", k), key_col),
                Span::styled(desc, desc_col),
            ]));
        }

        lines.push(Line::raw(""));

        // 2. Global shortcuts
        lines.push(Line::from(vec![Span::styled(
            match lang {
                AppLanguage::English => "[ Global Shortcuts ]",
                AppLanguage::Portuguese => "[ Atalhos Globais ]",
            },
            section_col,
        )]));

        let global_shortcuts: Vec<(&'static str, &'static str, &'static str)> = vec![
            ("[F2] / [Ctrl+S]", "Open Settings modal", "Abrir modal de Configurações"),
            ("[Ctrl+P] / [p]", "Select & install manga sources", "Selecionar e instalar fontes"),
            ("[?]", "Toggle this help modal", "Abrir / Fechar esta ajuda"),
            ("[Ctrl+C] / [q]", "Quit Neo-Mangal", "Sair do Neo-Mangal"),
        ];

        for (k, en_desc, pt_desc) in global_shortcuts {
            let desc = match lang {
                AppLanguage::English => en_desc,
                AppLanguage::Portuguese => pt_desc,
            };
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled(format!("{:<20}", k), key_col),
                Span::styled(desc, desc_col),
            ]));
        }

        frame.render_widget(Paragraph::new(lines), chunks[0]);

        // Footer
        let close_hint: Vec<Span> = match lang {
            AppLanguage::English => vec![
                Span::styled("[Esc / Enter / ?] ", key_col),
                Span::raw("Close Help"),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled("[Esc / Enter / ?] ", key_col),
                Span::raw("Fechar Ajuda"),
            ],
        };

        frame.render_widget(Paragraph::new(Line::from(close_hint)).alignment(Alignment::Center), chunks[1]);
    }
}
