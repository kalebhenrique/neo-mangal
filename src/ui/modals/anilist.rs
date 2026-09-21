use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;
use crate::ui::modals::settings::centered_rect;

pub struct AnilistModal {
    pub input_token: String,
    pub status_msg: Option<(String, Color)>,
    pub is_loading: bool,
}

impl AnilistModal {
    pub fn new() -> Self {
        Self {
            input_token: String::new(),
            status_msg: None,
            is_loading: false,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if !c.is_control() {
            self.input_token.push(c);
        }
    }

    pub fn handle_backspace(&mut self) {
        self.input_token.pop();
    }

    pub fn open_browser() {
        let url = "https://anilist.co/settings/developer";
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", "start", url]).spawn();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let modal_area = centered_rect(76, 58, area);
        frame.render_widget(Clear, modal_area);

        let border_color = if self.is_loading {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let title = match lang {
            AppLanguage::English => " ⚡ Connect AniList Account ",
            AppLanguage::Portuguese => " ⚡ Conectar Conta AniList ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(4), // Instructions / steps
                Constraint::Length(3), // Token input box
                Constraint::Length(2), // Status / Error message
                Constraint::Min(2),    // Action hints
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // 1. Steps instructions
        let instructions = match lang {
            AppLanguage::English => vec![
                Line::from(vec![
                    Span::styled("1. ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Create an API client at "),
                    Span::styled("https://anilist.co/settings/developer", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
                ]),
                Line::from(vec![
                    Span::raw("   Set "),
                    Span::styled("Redirect URL", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw(" to: "),
                    Span::styled("https://anilist.co/api/v2/oauth/pin", Style::default().fg(Color::LightGreen)),
                ]),
                Line::from(vec![
                    Span::styled("2. ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Paste your AniList Access Token below:"),
                ]),
            ],
            AppLanguage::Portuguese => vec![
                Line::from(vec![
                    Span::styled("1. ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Crie um cliente em "),
                    Span::styled("https://anilist.co/settings/developer", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED)),
                ]),
                Line::from(vec![
                    Span::raw("   Defina a "),
                    Span::styled("Redirect URL", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                    Span::raw(" como: "),
                    Span::styled("https://anilist.co/api/v2/oauth/pin", Style::default().fg(Color::LightGreen)),
                ]),
                Line::from(vec![
                    Span::styled("2. ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw("Cole seu Access Token do AniList abaixo:"),
                ]),
            ],
        };
        frame.render_widget(Paragraph::new(instructions), chunks[0]);

        // 2. Token Input box
        let input_title = match lang {
            AppLanguage::English => " AniList Access Token ",
            AppLanguage::Portuguese => " Access Token do AniList ",
        };
        let input_block = Block::default()
            .borders(Borders::ALL)
            .title(input_title)
            .border_style(Style::default().fg(Color::Yellow));

        let mut display_text = self.input_token.clone();
        if display_text.len() > 60 {
            // Mask or show tail so it fits nicely
            display_text = format!("...{}", &display_text[display_text.len() - 57..]);
        }

        let input_line = Line::from(vec![
            Span::styled(display_text, Style::default().fg(Color::White)),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]);
        frame.render_widget(Paragraph::new(input_line).block(input_block), chunks[1]);

        // 3. Status message
        if let Some((ref msg, color)) = self.status_msg {
            let status_line = Line::from(vec![Span::styled(
                msg.clone(),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )]);
            frame.render_widget(Paragraph::new(status_line), chunks[2]);
        } else if self.is_loading {
            let loading_text = match lang {
                AppLanguage::English => "⏳ Verifying token with AniList...",
                AppLanguage::Portuguese => "⏳ Verificando token com AniList...",
            };
            let status_line = Line::from(vec![Span::styled(
                loading_text,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            )]);
            frame.render_widget(Paragraph::new(status_line), chunks[2]);
        }

        // 4. Instructions
        let keys_data = match lang {
            AppLanguage::English => vec![
                ("[Enter] ", "Authenticate & Save   "),
                ("[Tab] ", "Open Browser   "),
                ("[Esc] ", "Cancel"),
            ],
            AppLanguage::Portuguese => vec![
                ("[Enter] ", "Autenticar e Salvar   "),
                ("[Tab] ", "Abrir no Navegador   "),
                ("[Esc] ", "Cancelar"),
            ],
        };

        let spans: Vec<Span> = keys_data
            .into_iter()
            .flat_map(|(key, desc)| {
                vec![
                    Span::styled(key, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    Span::raw(desc),
                ]
            })
            .collect();
        frame.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), chunks[3]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anilist_modal_input() {
        let mut modal = AnilistModal::new();
        assert_eq!(modal.input_token, "");
        modal.handle_char('a');
        modal.handle_char('b');
        assert_eq!(modal.input_token, "ab");
        modal.handle_backspace();
        assert_eq!(modal.input_token, "a");
    }
}
