use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::i18n::AppLanguage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnilistStep {
    ClientId,
    Token,
}

pub struct AnilistModal {
    pub client_id: String,
    pub token: String,
    pub step: AnilistStep,
    pub status_msg: Option<(String, Color)>,
    pub is_loading: bool,
}

impl AnilistModal {
    pub fn new() -> Self {
        Self {
            client_id: String::new(),
            token: String::new(),
            step: AnilistStep::ClientId,
            status_msg: None,
            is_loading: false,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        if c.is_control() {
            return;
        }
        match self.step {
            AnilistStep::ClientId => {
                self.client_id.push(c);
            }
            AnilistStep::Token => {
                self.token.push(c);
            }
        }
    }

    pub fn handle_backspace(&mut self) {
        match self.step {
            AnilistStep::ClientId => {
                self.client_id.pop();
            }
            AnilistStep::Token => {
                self.token.pop();
            }
        }
    }

    pub fn open_auth_browser(&self) {
        let trimmed = self.client_id.trim();
        let url = format!(
            "https://anilist.co/api/v2/oauth/authorize?client_id={}&response_type=token",
            trimmed
        );

        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&url).spawn();
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&url).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("cmd").args(["/C", "start", &url]).spawn();
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let border_color = if self.is_loading {
            Color::Yellow
        } else {
            Color::Cyan
        };

        let title = match lang {
            AppLanguage::English => " 󰑈 Connect AniList Account ",
            AppLanguage::Portuguese => " 󰑈 Conectar Conta AniList ",
        };

        match self.step {
            AnilistStep::ClientId => {
                let w = 68.min(area.width.saturating_sub(2));
                let h = 10.min(area.height.saturating_sub(2));
                let x = area.x + (area.width.saturating_sub(w)) / 2;
                let y = area.y + (area.height.saturating_sub(h)) / 2;
                let modal_area = Rect { x, y, width: w, height: h };

                frame.render_widget(Clear, modal_area);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

                let inner_area = block.inner(modal_area);
                frame.render_widget(block, modal_area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([
                        Constraint::Length(1), // Clickable Link line
                        Constraint::Length(3), // Client ID Input Box
                        Constraint::Length(1), // Status message
                        Constraint::Min(1),    // Footer hints
                    ])
                    .split(inner_area);

                // 1. Clickable Link
                let link_line = Line::from(vec![
                    Span::styled(
                        match lang {
                            AppLanguage::English => "1. Create client: ",
                            AppLanguage::Portuguese => "1. Crie seu client: ",
                        },
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        "https://anilist.co/settings/developer",
                        Style::default()
                            .fg(Color::LightCyan)
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                ]);
                frame.render_widget(Paragraph::new(link_line), chunks[0]);

                // 2. Client ID Box
                let id_block = Block::default()
                    .borders(Borders::ALL)
                    .title(match lang {
                        AppLanguage::English => " Client ID ",
                        AppLanguage::Portuguese => " Client ID ",
                    })
                    .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

                let id_line = Line::from(vec![
                    Span::styled(&self.client_id, Style::default().fg(Color::White)),
                    Span::styled("█", Style::default().fg(Color::Yellow)),
                ]);
                frame.render_widget(Paragraph::new(id_line).block(id_block), chunks[1]);

                // 3. Status message
                if let Some((ref msg, color)) = self.status_msg {
                    let status_line = Line::from(vec![Span::styled(
                        msg.clone(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )]);
                    frame.render_widget(Paragraph::new(status_line), chunks[2]);
                }

                // 4. Footer hints
                let has_id = !self.client_id.trim().is_empty();
                let keys_data: Vec<(&str, &str)> = if has_id {
                    match lang {
                        AppLanguage::English => vec![
                            ("[Enter] ", "Open Browser   "),
                            ("[Esc] ", "Cancel"),
                        ],
                        AppLanguage::Portuguese => vec![
                            ("[Enter] ", "Abrir Navegador   "),
                            ("[Esc] ", "Cancelar"),
                        ],
                    }
                } else {
                    match lang {
                        AppLanguage::English => vec![("[Esc] ", "Cancel")],
                        AppLanguage::Portuguese => vec![("[Esc] ", "Cancelar")],
                    }
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
            AnilistStep::Token => {
                let w = 68.min(area.width.saturating_sub(2));
                let h = 11.min(area.height.saturating_sub(2));
                let x = area.x + (area.width.saturating_sub(w)) / 2;
                let y = area.y + (area.height.saturating_sub(h)) / 2;
                let modal_area = Rect { x, y, width: w, height: h };

                frame.render_widget(Clear, modal_area);

                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

                let inner_area = block.inner(modal_area);
                frame.render_widget(block, modal_area);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([
                        Constraint::Length(1), // Step 1 confirmed: Client ID
                        Constraint::Length(1), // Step 2 Header
                        Constraint::Length(3), // Token Input Box
                        Constraint::Length(1), // Status / Loading message
                        Constraint::Min(1),    // Footer hints
                    ])
                    .split(inner_area);

                // 1. Client ID Confirmed
                let confirmed_line = Line::from(vec![
                    Span::styled("✓ Client ID: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled(&self.client_id, Style::default().fg(Color::White)),
                ]);
                frame.render_widget(Paragraph::new(confirmed_line), chunks[0]);

                // 2. Step 2 Instruction
                let step2_text = match lang {
                    AppLanguage::English => "2. Paste generated Access Token from browser:",
                    AppLanguage::Portuguese => "2. Cole o Access Token gerado no navegador:",
                };
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(step2_text, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                    ])),
                    chunks[1],
                );

                // 3. Token Input Box
                let token_block = Block::default()
                    .borders(Borders::ALL)
                    .title(match lang {
                        AppLanguage::English => " Access Token ",
                        AppLanguage::Portuguese => " Access Token ",
                    })
                    .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

                let mut display_token = self.token.clone();
                if display_token.len() > 50 {
                    display_token = format!("...{}", &display_token[display_token.len() - 47..]);
                }
                let token_line = Line::from(vec![
                    Span::styled(display_token, Style::default().fg(Color::White)),
                    Span::styled("█", Style::default().fg(Color::Yellow)),
                ]);
                frame.render_widget(Paragraph::new(token_line).block(token_block), chunks[2]);

                // 4. Status message
                if let Some((ref msg, color)) = self.status_msg {
                    let status_line = Line::from(vec![Span::styled(
                        msg.clone(),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    )]);
                    frame.render_widget(Paragraph::new(status_line), chunks[3]);
                } else if self.is_loading {
                    let loading_text = match lang {
                        AppLanguage::English => "󱑂 Verifying token with AniList...",
                        AppLanguage::Portuguese => "󱑂 Verificando token com AniList...",
                    };
                    let status_line = Line::from(vec![Span::styled(
                        loading_text,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    )]);
                    frame.render_widget(Paragraph::new(status_line), chunks[3]);
                }

                // 5. Action hints footer
                let keys_data = match lang {
                    AppLanguage::English => vec![
                        ("[Enter] ", "Connect & Save   "),
                        ("[Esc] ", "Back"),
                    ],
                    AppLanguage::Portuguese => vec![
                        ("[Enter] ", "Conectar e Salvar   "),
                        ("[Esc] ", "Voltar"),
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
                frame.render_widget(Paragraph::new(Line::from(spans)).alignment(Alignment::Center), chunks[4]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anilist_modal_flow() {
        let mut modal = AnilistModal::new();
        assert_eq!(modal.step, AnilistStep::ClientId);
        assert!(modal.client_id.is_empty());

        modal.handle_char('1');
        modal.handle_char('2');
        assert_eq!(modal.client_id, "12");

        modal.step = AnilistStep::Token;
        modal.handle_char('t');
        modal.handle_char('o');
        modal.handle_char('k');
        assert_eq!(modal.token, "tok");

        modal.handle_backspace();
        assert_eq!(modal.token, "to");
    }
}
