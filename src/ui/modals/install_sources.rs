use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use crate::i18n::AppLanguage;
use crate::scraper::manager::SourceManager;
use crate::ui::modals::settings::centered_rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallState {
    Prompt,
    Downloading,
    Success(usize),
    Error(String),
}

pub struct InstallSourcesModal {
    pub state: InstallState,
}

impl Default for InstallSourcesModal {
    fn default() -> Self {
        Self::new()
    }
}

impl InstallSourcesModal {
    pub fn new() -> Self {
        Self {
            state: InstallState::Prompt,
        }
    }

    pub fn execute_install(&mut self) -> Result<Vec<String>, String> {
        self.state = InstallState::Downloading;

        // Check if user has local clone in ~/work/neo-mangal-scrapers/scrapers
        let home_work = dirs::home_dir()
            .map(|h| h.join("work").join("neo-mangal-scrapers").join("scrapers"))
            .filter(|p| p.exists());

        if let Some(local_dir) = home_work {
            let target_dir = SourceManager::ensure_sources_dir().map_err(|e| e.to_string())?;
            let mut installed = Vec::new();
            if let Ok(entries) = std::fs::read_dir(local_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                        if let Some(file_name) = path.file_name() {
                            let dest = target_dir.join(file_name);
                            if let Ok(_) = std::fs::copy(&path, &dest) {
                                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                                    installed.push(stem.to_string());
                                }
                            }
                        }
                    }
                }
            }
            if !installed.is_empty() {
                installed.sort();
                self.state = InstallState::Success(installed.len());
                return Ok(installed);
            }
        }

        // Otherwise download from official GitHub repository
        match SourceManager::install_sources_from_repo("kalebhenrique/neo-mangal-scrapers") {
            Ok(sources) => {
                self.state = InstallState::Success(sources.len());
                Ok(sources)
            }
            Err(e) => {
                self.state = InstallState::Error(e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let modal_area = centered_rect(65, 48, area);
        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰋩 Manga Sources Setup ",
            AppLanguage::Portuguese => " 󰋩 Configuração de Fontes de Mangá ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Header title
                Constraint::Min(4),    // Description / body
                Constraint::Length(3), // Actions
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // 1. Header
        let head_text = match lang {
            AppLanguage::English => "No manga scrapers detected in your system",
            AppLanguage::Portuguese => "Nenhum scraper de mangá detectado no seu sistema",
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled("󰋩 ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(head_text, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            ]))
            .alignment(Alignment::Center),
            chunks[0],
        );

        // 2. Body based on state
        let body_lines = match &self.state {
            InstallState::Prompt => match lang {
                AppLanguage::English => vec![
                    Line::from("neo-mangal uses external Lua scrapers to ensure resilience,"),
                    Line::from("faster updates, and protection against DMCA notices."),
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("Would you like to install official scrapers from "),
                        Span::styled(
                            "kalebhenrique/neo-mangal-scrapers",
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::raw("?"),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Includes: WeebCentral, MangaDex, MangaKakalot, Manganelo, Mangasee, etc.",
                        Style::default().fg(Color::DarkGray),
                    )),
                ],
                AppLanguage::Portuguese => vec![
                    Line::from("O neo-mangal utiliza scrapers externos em Lua para garantir"),
                    Line::from("atualizações ágeis e proteção jurídica contra DMCA."),
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("Deseja baixar e instalar os scrapers oficiais de "),
                        Span::styled(
                            "kalebhenrique/neo-mangal-scrapers",
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::raw("?"),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Inclui: WeebCentral, MangaDex, MangaKakalot, Manganelo, Mangasee, etc.",
                        Style::default().fg(Color::DarkGray),
                    )),
                ],
            },
            InstallState::Downloading => match lang {
                AppLanguage::English => vec![
                    Line::from(""),
                    Line::from(Span::styled("󰑮 Downloading official scrapers from GitHub...", Style::default().fg(Color::Yellow))),
                    Line::from(""),
                    Line::from(Span::styled("Saving to ~/.config/neo-mangal/sources/", Style::default().fg(Color::DarkGray))),
                ],
                AppLanguage::Portuguese => vec![
                    Line::from(""),
                    Line::from(Span::styled("󰑮 Baixando scrapers oficiais do GitHub...", Style::default().fg(Color::Yellow))),
                    Line::from(""),
                    Line::from(Span::styled("Salvando em ~/.config/neo-mangal/sources/", Style::default().fg(Color::DarkGray))),
                ],
            },
            InstallState::Success(count) => match lang {
                AppLanguage::English => vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("󰄬 Successfully installed {} scrapers!", count),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("You can now search and download manga with neo-mangal.", Style::default().fg(Color::White))),
                ],
                AppLanguage::Portuguese => vec![
                    Line::from(""),
                    Line::from(Span::styled(
                        format!("󰄬 {} scrapers instalados com sucesso!", count),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(Span::styled("Agora você pode pesquisar e baixar mangás normalmente.", Style::default().fg(Color::White))),
                ],
            },
            InstallState::Error(err) => match lang {
                AppLanguage::English => vec![
                    Line::from(Span::styled("󰀦 Failed to download scrapers:", Style::default().fg(Color::Red))),
                    Line::from(Span::styled(err, Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                    Line::from("You can run 'neo-mangal sources install' later via terminal."),
                ],
                AppLanguage::Portuguese => vec![
                    Line::from(Span::styled("󰀦 Falha ao baixar scrapers:", Style::default().fg(Color::Red))),
                    Line::from(Span::styled(err, Style::default().fg(Color::DarkGray))),
                    Line::from(""),
                    Line::from("Você pode rodar 'neo-mangal sources install' mais tarde pelo terminal."),
                ],
            },
        };

        frame.render_widget(Paragraph::new(body_lines).alignment(Alignment::Center), chunks[1]);

        // 3. Actions
        let action_spans = match (&self.state, lang) {
            (InstallState::Prompt, AppLanguage::English) => vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Download & Install    "),
                Span::styled(" [Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Skip"),
            ],
            (InstallState::Prompt, AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Baixar e Instalar    "),
                Span::styled(" [Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Pular"),
            ],
            (InstallState::Downloading, AppLanguage::English) => vec![
                Span::styled("Please wait...", Style::default().fg(Color::Yellow)),
            ],
            (InstallState::Downloading, AppLanguage::Portuguese) => vec![
                Span::styled("Aguarde...", Style::default().fg(Color::Yellow)),
            ],
            (InstallState::Success(_), AppLanguage::English) => vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Continue"),
            ],
            (InstallState::Success(_), AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter] ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                Span::raw("Continuar"),
            ],
            (InstallState::Error(_), AppLanguage::English) => vec![
                Span::styled(" [Enter / Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Close"),
            ],
            (InstallState::Error(_), AppLanguage::Portuguese) => vec![
                Span::styled(" [Enter / Esc] ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                Span::raw("Fechar"),
            ],
        };

        frame.render_widget(
            Paragraph::new(Line::from(action_spans)).alignment(Alignment::Center),
            chunks[2],
        );
    }
}
