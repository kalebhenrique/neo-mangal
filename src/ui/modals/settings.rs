use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use std::path::Path;
use crate::config::Config;
use crate::i18n::{AppLanguage, I18n};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingItem {
    DownloadDir,
    Language,
    KindleProfile,
    OutputFormat,
    AnilistAccount,
    AnilistAutoSync,
}

impl SettingItem {
    pub fn next(self) -> Self {
        match self {
            SettingItem::DownloadDir => SettingItem::Language,
            SettingItem::Language => SettingItem::KindleProfile,
            SettingItem::KindleProfile => SettingItem::OutputFormat,
            SettingItem::OutputFormat => SettingItem::AnilistAccount,
            SettingItem::AnilistAccount => SettingItem::AnilistAutoSync,
            SettingItem::AnilistAutoSync => SettingItem::DownloadDir,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            SettingItem::DownloadDir => SettingItem::AnilistAutoSync,
            SettingItem::Language => SettingItem::DownloadDir,
            SettingItem::KindleProfile => SettingItem::Language,
            SettingItem::OutputFormat => SettingItem::KindleProfile,
            SettingItem::AnilistAccount => SettingItem::OutputFormat,
            SettingItem::AnilistAutoSync => SettingItem::AnilistAccount,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsAction {
    EditDownloadPath,
    OpenAnilist,
    DisconnectAnilist,
}

pub struct SettingsModal {
    pub input_path: String,
    pub language: String,
    pub profile: String,
    pub format: String,
    pub anilist_username: Option<String>,
    pub anilist_enabled: bool,
    pub anilist_sync_on_download: bool,
    pub selected_item: SettingItem,
}

impl SettingsModal {
    pub fn new(config: &Config) -> Self {
        Self {
            input_path: config.download_dir.to_string_lossy().to_string(),
            language: config.language.clone(),
            profile: config.kcc_profile.clone(),
            format: config.kcc_format.clone(),
            anilist_username: config.anilist_username.clone(),
            anilist_enabled: config.anilist_enabled,
            anilist_sync_on_download: config.anilist_sync_on_download,
            selected_item: SettingItem::DownloadDir,
        }
    }

    pub fn current_lang(&self) -> AppLanguage {
        AppLanguage::from_code(&self.language)
    }

    pub fn next_item(&mut self) {
        self.selected_item = self.selected_item.next();
    }

    pub fn prev_item(&mut self) {
        self.selected_item = self.selected_item.prev();
    }

    pub fn cycle_language(&mut self) {
        match self.current_lang() {
            AppLanguage::English => {
                self.language = "pt-br".to_string();
            }
            AppLanguage::Portuguese => {
                self.language = "en".to_string();
            }
        }
    }

    #[allow(dead_code)]
    pub fn cycle_profile(&mut self) {
        self.cycle_profile_forward();
    }

    pub fn cycle_profile_forward(&mut self) {
        let profiles = ["KPW5", "KV", "KO", "KS", "K11", "KPW"];
        if let Some(pos) = profiles.iter().position(|&p| p == self.profile) {
            let next_idx = (pos + 1) % profiles.len();
            self.profile = profiles[next_idx].to_string();
        } else {
            self.profile = "KPW5".to_string();
        }
    }

    pub fn cycle_profile_backward(&mut self) {
        let profiles = ["KPW5", "KV", "KO", "KS", "K11", "KPW"];
        if let Some(pos) = profiles.iter().position(|&p| p == self.profile) {
            let prev_idx = if pos == 0 { profiles.len() - 1 } else { pos - 1 };
            self.profile = profiles[prev_idx].to_string();
        } else {
            self.profile = "KPW".to_string();
        }
    }

    #[allow(dead_code)]
    pub fn cycle_format(&mut self) {
        self.cycle_format_forward();
    }

    pub fn cycle_format_forward(&mut self) {
        let formats = ["AZW3", "CBZ", "EPUB", "MOBI", "PDF"];
        if let Some(pos) = formats.iter().position(|&f| f.eq_ignore_ascii_case(&self.format)) {
            let next_idx = (pos + 1) % formats.len();
            self.format = formats[next_idx].to_string();
        } else {
            self.format = "AZW3".to_string();
        }
    }

    pub fn cycle_format_backward(&mut self) {
        let formats = ["AZW3", "CBZ", "EPUB", "MOBI", "PDF"];
        if let Some(pos) = formats.iter().position(|&f| f.eq_ignore_ascii_case(&self.format)) {
            let prev_idx = if pos == 0 { formats.len() - 1 } else { pos - 1 };
            self.format = formats[prev_idx].to_string();
        } else {
            self.format = "PDF".to_string();
        }
    }

    pub fn toggle_sync_on_download(&mut self) {
        self.anilist_sync_on_download = !self.anilist_sync_on_download;
    }

    pub fn disconnect_anilist(&mut self) {
        self.anilist_username = None;
        self.anilist_enabled = false;
    }

    pub fn handle_enter(&mut self) -> Option<SettingsAction> {
        match self.selected_item {
            SettingItem::DownloadDir => Some(SettingsAction::EditDownloadPath),
            SettingItem::Language => {
                self.cycle_language();
                None
            }
            SettingItem::KindleProfile => {
                self.cycle_profile_forward();
                None
            }
            SettingItem::OutputFormat => {
                self.cycle_format_forward();
                None
            }
            SettingItem::AnilistAccount => {
                if self.anilist_username.is_some() {
                    self.disconnect_anilist();
                    Some(SettingsAction::DisconnectAnilist)
                } else {
                    Some(SettingsAction::OpenAnilist)
                }
            }
            SettingItem::AnilistAutoSync => {
                if self.anilist_username.is_some() {
                    self.toggle_sync_on_download();
                    None
                } else {
                    Some(SettingsAction::OpenAnilist)
                }
            }
        }
    }

    pub fn handle_space(&mut self) -> Option<SettingsAction> {
        self.handle_enter()
    }

    pub fn handle_left(&mut self) {
        match self.selected_item {
            SettingItem::Language => self.cycle_language(),
            SettingItem::KindleProfile => self.cycle_profile_backward(),
            SettingItem::OutputFormat => self.cycle_format_backward(),
            _ => {}
        }
    }

    pub fn handle_right(&mut self) {
        match self.selected_item {
            SettingItem::Language => self.cycle_language(),
            SettingItem::KindleProfile => self.cycle_profile_forward(),
            SettingItem::OutputFormat => self.cycle_format_forward(),
            _ => {}
        }
    }


    pub fn profile_short_description(profile: &str) -> &'static str {
        match profile {
            "KPW5" => "Paperwhite 11ª/12ª Gen",
            "KV" => "Paperwhite 3/4 / Voyage",
            "KO" => "Oasis 2ª/3ª Gen",
            "KS" => "Kindle Scribe (10.2\")",
            "K11" => "Kindle Básico (11ª Gen)",
            "KPW" => "Kindle Paperwhite Antigo",
            _ => "Kindle Device",
        }
    }

    pub fn profile_full_description(profile: &str, lang: AppLanguage) -> &'static str {
        match (profile, lang) {
            ("KPW5", AppLanguage::English) => "Kindle Paperwhite (11th / 12th Gen - 300 ppi, 1236x1648)",
            ("KPW5", AppLanguage::Portuguese) => "Kindle Paperwhite (11ª / 12ª Geração - 300 ppi, 1236x1648)",
            ("KV", AppLanguage::English) => "Kindle Paperwhite (3rd / 4th Gen) / Voyage (300 ppi, 1072x1448)",
            ("KV", AppLanguage::Portuguese) => "Kindle Paperwhite (3ª / 4ª Geração) / Voyage (300 ppi, 1072x1448)",
            ("KO", AppLanguage::English) => "Kindle Oasis (2nd / 3rd Gen - 300 ppi, 1264x1680)",
            ("KO", AppLanguage::Portuguese) => "Kindle Oasis (2ª / 3ª Geração - 300 ppi, 1264x1680)",
            ("KS", AppLanguage::English) => "Kindle Scribe (10.2\" - 300 ppi, 1860x2480)",
            ("KS", AppLanguage::Portuguese) => "Kindle Scribe (10.2\" - 300 ppi, 1860x2480)",
            ("K11", AppLanguage::English) => "Kindle Basic (11th Gen - 300 ppi, 1072x1448)",
            ("K11", AppLanguage::Portuguese) => "Kindle Básico (11ª Geração - 300 ppi, 1072x1448)",
            ("KPW", AppLanguage::English) => "Kindle Paperwhite (Legacy 1st / 2nd Gen - 212 ppi, 758x1024)",
            ("KPW", AppLanguage::Portuguese) => "Kindle Paperwhite (1ª / 2ª Geração antiga - 212 ppi, 758x1024)",
            _ => "Kindle Device",
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lang = self.current_lang();

        // Responsive size dynamically calculated based on terminal window:
        // ~82% width (min 68, max 96 cols, capped by available width)
        let w = ((area.width as u32 * 82) / 100).clamp(68, 96) as u16;
        let w = w.min(area.width.saturating_sub(2));

        // ~78% height (min 15, max 26 rows, capped by available height)
        let h = ((area.height as u32 * 78) / 100).clamp(15, 26) as u16;
        let h = h.min(area.height.saturating_sub(2));

        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;
        let modal_area = Rect { x, y, width: w, height: h };

        frame.render_widget(Clear, modal_area);

        let border_color = Color::Yellow;

        let title = match lang {
            AppLanguage::English => " 󰒓 Settings ",
            AppLanguage::Portuguese => " 󰒓 Configurações ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(border_color).add_modifier(Modifier::BOLD));

        let inner_area = block.inner(modal_area);
        frame.render_widget(block, modal_area);

        let has_info_box = h >= 19;
        let chunks = if has_info_box {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(11),   // Settings items
                    Constraint::Length(4), // Dedicated Info card
                    Constraint::Length(1), // Footer key hints
                ])
                .split(inner_area)
        } else {
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Min(9),    // Settings items + 1 line help
                    Constraint::Length(1), // Footer key hints
                ])
                .split(inner_area)
        };

        let mut lines = Vec::new();

        if h >= 22 {
            lines.push(Line::raw(""));
        }

        // ─── Section 1: General ───
        lines.push(Line::from(vec![Span::styled(
            match lang {
                AppLanguage::English => "[ General ]",
                AppLanguage::Portuguese => "[ Geral ]",
            },
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]));

        // 1. Download Directory
        {
            let is_sel = self.selected_item == SettingItem::DownloadDir;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };

            let label_text = match lang {
                AppLanguage::English => "Download Dir:    ",
                AppLanguage::Portuguese => "Pasta Download:  ",
            };

            let spans = vec![
                Span::styled(prefix, pref_style),
                Span::styled(label_text, label_style),
                Span::styled(&self.input_path, Style::default().fg(Color::Cyan)),
            ];
            lines.push(Line::from(spans));
        }

        // 2. Language
        {
            let is_sel = self.selected_item == SettingItem::Language;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let label = match lang {
                AppLanguage::English => "Language:        ",
                AppLanguage::Portuguese => "Idioma:          ",
            };
            let lang_display = match self.language.as_str() {
                "pt-br" => "Português (Brasil)",
                _ => "English",
            };
            lines.push(Line::from(vec![
                Span::styled(prefix, pref_style),
                Span::styled(label, label_style),
                Span::styled(
                    format!("[ {} ]", lang_display),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if h >= 18 {
            lines.push(Line::raw(""));
        }

        // ─── Section 2: Kindle & Converter ───
        lines.push(Line::from(vec![Span::styled(
            match lang {
                AppLanguage::English => "[ Kindle & Converter ]",
                AppLanguage::Portuguese => "[ Kindle & Conversão ]",
            },
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]));

        // 3. Kindle Model
        {
            let is_sel = self.selected_item == SettingItem::KindleProfile;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let label = match lang {
                AppLanguage::English => "Kindle Model:    ",
                AppLanguage::Portuguese => "Modelo Kindle:   ",
            };
            let desc = Self::profile_short_description(&self.profile);
            lines.push(Line::from(vec![
                Span::styled(prefix, pref_style),
                Span::styled(label, label_style),
                Span::styled(
                    format!("[ {} ]", self.profile),
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ),
                Span::raw(" "),
                Span::styled(desc, Style::default().fg(Color::White)),
            ]));
        }

        // 4. Output Format
        {
            let is_sel = self.selected_item == SettingItem::OutputFormat;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let label = match lang {
                AppLanguage::English => "Output Format:   ",
                AppLanguage::Portuguese => "Formato Saída:   ",
            };
            lines.push(Line::from(vec![
                Span::styled(prefix, pref_style),
                Span::styled(label, label_style),
                Span::styled(
                    format!("[ {} ]", self.format),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                ),
            ]));
        }

        if h >= 18 {
            lines.push(Line::raw(""));
        }

        // ─── Section 3: AniList ───
        lines.push(Line::from(vec![Span::styled(
            "[ AniList ]",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )]));

        // 5. AniList Account
        {
            let is_sel = self.selected_item == SettingItem::AnilistAccount;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let label = match lang {
                AppLanguage::English => "Account:         ",
                AppLanguage::Portuguese => "Conta AniList:   ",
            };

            let line_account = if let Some(ref user) = self.anilist_username {
                Line::from(vec![
                    Span::styled(prefix, pref_style),
                    Span::styled(label, label_style),
                    Span::styled(
                        format!("@{}", user),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        match lang {
                            AppLanguage::English => " (Connected)",
                            AppLanguage::Portuguese => " (Conectado)",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            } else {
                let prompt = match lang {
                    AppLanguage::English => "[ Connect Account ]",
                    AppLanguage::Portuguese => "[ Conectar Conta ]",
                };
                Line::from(vec![
                    Span::styled(prefix, pref_style),
                    Span::styled(label, label_style),
                    Span::styled(
                        match lang {
                            AppLanguage::English => "Disconnected  ",
                            AppLanguage::Portuguese => "Desconectado  ",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(prompt, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ])
            };
            lines.push(line_account);
        }

        // 6. AniList Auto-Sync on Download
        {
            let is_sel = self.selected_item == SettingItem::AnilistAutoSync;
            let prefix = if is_sel { " ► " } else { "   " };
            let pref_style = if is_sel {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let label_style = if is_sel {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Gray)
            };
            let label = match lang {
                AppLanguage::English => "Auto-Sync:       ",
                AppLanguage::Portuguese => "Auto-Sync:       ",
            };

            let line_sync = if self.anilist_username.is_some() {
                let (status_tag, status_col) = if self.anilist_sync_on_download {
                    (
                        match lang {
                            AppLanguage::English => "[ ENABLED ]",
                            AppLanguage::Portuguese => "[ ATIVADO ]",
                        },
                        Color::Green,
                    )
                } else {
                    (
                        match lang {
                            AppLanguage::English => "[ DISABLED ]",
                            AppLanguage::Portuguese => "[ DESATIVADO ]",
                        },
                        Color::DarkGray,
                    )
                };
                Line::from(vec![
                    Span::styled(prefix, pref_style),
                    Span::styled(label, label_style),
                    Span::styled(status_tag, Style::default().fg(status_col).add_modifier(Modifier::BOLD)),
                ])
            } else {
                Line::from(vec![
                    Span::styled(prefix, pref_style),
                    Span::styled(label, label_style),
                    Span::styled(
                        match lang {
                            AppLanguage::English => "[ DISABLED ] (Requires AniList login)",
                            AppLanguage::Portuguese => "[ DESATIVADO ] (Requer login AniList)",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            };
            lines.push(line_sync);
        }

        // Info lines for contextual help / dedicated Info box
        let (info_line1, info_line2) = match self.selected_item {
            SettingItem::DownloadDir => {
                let path_ref = Path::new(&self.input_path);
                let status_span = if path_ref.is_dir() {
                    Span::styled(I18n::dir_valid(lang), Style::default().fg(Color::Green))
                } else if !self.input_path.trim().is_empty() {
                    Span::styled(I18n::dir_will_create(lang), Style::default().fg(Color::Yellow))
                } else {
                    Span::styled(I18n::dir_empty_err(lang), Style::default().fg(Color::Red))
                };
                (
                    Line::from(vec![
                        Span::styled("Status: ", Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        status_span,
                    ]),
                    Line::from(vec![
                        Span::styled(
                            match lang {
                                AppLanguage::English => "Press Enter to edit path with [Tab] autocomplete support.",
                                AppLanguage::Portuguese => "Pressione Enter para editar o caminho com suporte a autocompletar [Tab].",
                            },
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]),
                )
            }
            SettingItem::Language => (
                Line::from(vec![
                    Span::styled(
                        match lang {
                            AppLanguage::English => "Interface Language: English / Português (Brasil)",
                            AppLanguage::Portuguese => "Idioma da Interface: Português (Brasil) / English",
                        },
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        match lang {
                            AppLanguage::English => "Switches all interface labels, alerts, and navigation text.",
                            AppLanguage::Portuguese => "Altera todos os textos da interface, avisos e navegação.",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ),
            SettingItem::KindleProfile => (
                Line::from(vec![
                    Span::styled(
                        Self::profile_full_description(&self.profile, lang),
                        Style::default().fg(Color::LightGreen),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        match lang {
                            AppLanguage::English => "KCC optimizes image dimensions and contrast for this e-ink device screen.",
                            AppLanguage::Portuguese => "O KCC otimiza resolução e contraste da imagem para a tela deste Kindle.",
                        },
                        Style::default().fg(Color::DarkGray),
                    ),
                ]),
            ),
            SettingItem::OutputFormat => {
                let format_desc = match self.format.to_uppercase().as_str() {
                    "AZW3" => match lang {
                        AppLanguage::English => "Kindle KF8 format. Recommended for all modern Kindle devices.",
                        AppLanguage::Portuguese => "Formato Kindle KF8. Recomendado para todos os Kindles modernos.",
                    },
                    "CBZ" => match lang {
                        AppLanguage::English => "Comic Book ZIP archive containing original chapter images.",
                        AppLanguage::Portuguese => "Arquivo ZIP contendo as imagens originais do capítulo.",
                    },
                    "EPUB" => match lang {
                        AppLanguage::English => "Universal e-book format for Kobo, tablets, and generic readers.",
                        AppLanguage::Portuguese => "Formato padrão de e-book para Kobo, tablets e outros leitores.",
                    },
                    "MOBI" => match lang {
                        AppLanguage::English => "Legacy Kindle format for older Kindle models.",
                        AppLanguage::Portuguese => "Formato Kindle legado para modelos antigos.",
                    },
                    "PDF" => match lang {
                        AppLanguage::English => "Portable Document Format. Universal viewable document on any device.",
                        AppLanguage::Portuguese => "Documento em formato PDF. Visualização universal em qualquer dispositivo.",
                    },
                    _ => "",
                };
                (
                    Line::from(vec![
                        Span::styled(
                            match lang {
                                AppLanguage::English => "Output Format: AZW3 (Kindle), CBZ (Archive), EPUB (Standard), MOBI (Legacy), PDF (Document)",
                                AppLanguage::Portuguese => "Formato: AZW3 (Kindle), CBZ (Arquivo), EPUB (Padrão), MOBI (Legado), PDF (Documento)",
                            },
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(format_desc, Style::default().fg(Color::DarkGray)),
                    ]),
                )
            }
            SettingItem::AnilistAccount => {
                if let Some(ref user) = self.anilist_username {
                    (
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => format!("Connected as @{}. Reading progress and favorites are synced.", user),
                                    AppLanguage::Portuguese => format!("Conectado como @{}. Progresso de leitura e favoritos sincronizados.", user),
                                },
                                Style::default().fg(Color::Green),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Press Enter to disconnect this AniList account.",
                                    AppLanguage::Portuguese => "Pressione Enter para desconectar esta conta do AniList.",
                                },
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]),
                    )
                } else {
                    (
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Connect your AniList account to track read manga chapters.",
                                    AppLanguage::Portuguese => "Conecte sua conta AniList para registrar capítulos e mangás lidos.",
                                },
                                Style::default().fg(Color::White),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Press Enter to open the AniList authentication wizard.",
                                    AppLanguage::Portuguese => "Pressione Enter para abrir o assistente de conexão do AniList.",
                                },
                                Style::default().fg(Color::Cyan),
                            ),
                        ]),
                    )
                }
            }
            SettingItem::AnilistAutoSync => {
                if self.anilist_username.is_some() {
                    (
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Automatically updates chapter progress on AniList upon download.",
                                    AppLanguage::Portuguese => "Atualiza o progresso no AniList automaticamente ao baixar capítulos.",
                                },
                                Style::default().fg(if self.anilist_sync_on_download { Color::Green } else { Color::White }),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Synchronizes reading status directly with your AniList library.",
                                    AppLanguage::Portuguese => "Sincroniza status e capítulos lidos diretamente na sua conta AniList.",
                                },
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]),
                    )
                } else {
                    (
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Auto-sync requires an active AniList account connection.",
                                    AppLanguage::Portuguese => "O auto-sync requer login ativo em uma conta do AniList.",
                                },
                                Style::default().fg(Color::Yellow),
                            ),
                        ]),
                        Line::from(vec![
                            Span::styled(
                                match lang {
                                    AppLanguage::English => "Connect your AniList account above to enable automatic progress tracking.",
                                    AppLanguage::Portuguese => "Conecte sua conta AniList acima para ativar a sincronização automática.",
                                },
                                Style::default().fg(Color::DarkGray),
                            ),
                        ]),
                    )
                }
            }
        };

        if has_info_box {
            frame.render_widget(Paragraph::new(lines), chunks[0]);

            let info_block = Block::default()
                .borders(Borders::ALL)
                .title(match lang {
                    AppLanguage::English => " ℹ Info ",
                    AppLanguage::Portuguese => " ℹ Detalhes ",
                })
                .border_style(Style::default().fg(Color::DarkGray));
            frame.render_widget(Paragraph::new(vec![info_line1, info_line2]).block(info_block), chunks[1]);
        } else {
            lines.push(info_line1);
            frame.render_widget(Paragraph::new(lines), chunks[0]);
        }

        // Footer hints
        let footer_spans: Vec<Span> = match lang {
            AppLanguage::English => vec![
                Span::styled("[↑/↓] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Navigate   "),
                Span::styled("[Enter/Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Change   "),
                Span::styled("[Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Save & Close"),
            ],
            AppLanguage::Portuguese => vec![
                Span::styled("[↑/↓] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Navegar   "),
                Span::styled("[Enter/Espaço] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Alterar   "),
                Span::styled("[Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::raw("Salvar e Fechar"),
            ],
        };
        let footer_idx = if has_info_box { 2 } else { 1 };
        frame.render_widget(Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center), chunks[footer_idx]);
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_and_toggle() {
        let config = Config::default();
        let mut modal = SettingsModal::new(&config);

        assert_eq!(modal.selected_item, SettingItem::DownloadDir);
        modal.next_item();
        assert_eq!(modal.selected_item, SettingItem::Language);
        modal.next_item();
        assert_eq!(modal.selected_item, SettingItem::KindleProfile);
        modal.prev_item();
        assert_eq!(modal.selected_item, SettingItem::Language);

        // Language toggle
        assert_eq!(modal.language, "en");
        modal.handle_enter();
        assert_eq!(modal.language, "pt-br");

        // Profile navigation and cycling
        modal.selected_item = SettingItem::KindleProfile;
        assert_eq!(modal.profile, "KPW5");
        modal.cycle_profile();
        assert_eq!(modal.profile, "KV");
        modal.handle_right();
        assert_eq!(modal.profile, "KO");
        modal.handle_left();
        assert_eq!(modal.profile, "KV");

        // Format cycling
        modal.selected_item = SettingItem::OutputFormat;
        assert_eq!(modal.format, "AZW3");
        modal.handle_enter();
        assert_eq!(modal.format, "CBZ");

        // Auto-sync toggle when connected
        modal.anilist_username = Some("testuser".to_string());
        modal.selected_item = SettingItem::AnilistAutoSync;
        assert!(!modal.anilist_sync_on_download);
        modal.handle_space();
        assert!(modal.anilist_sync_on_download);
        modal.handle_space();
        assert!(!modal.anilist_sync_on_download);
    }

    #[test]
    fn test_cycle_format() {
        let config = Config::default();
        let mut modal = SettingsModal::new(&config);
        assert_eq!(modal.format, "AZW3");

        modal.cycle_format();
        assert_eq!(modal.format, "CBZ");

        modal.cycle_format();
        assert_eq!(modal.format, "EPUB");

        modal.cycle_format();
        assert_eq!(modal.format, "MOBI");

        modal.cycle_format();
        assert_eq!(modal.format, "PDF");

        modal.cycle_format();
        assert_eq!(modal.format, "AZW3");
    }

    #[test]
    fn test_download_dir_edit_action() {
        let config = Config::default();
        let mut modal = SettingsModal::new(&config);
        modal.selected_item = SettingItem::DownloadDir;
        assert_eq!(modal.handle_enter(), Some(SettingsAction::EditDownloadPath));
        assert_eq!(modal.handle_space(), Some(SettingsAction::EditDownloadPath));
    }
}

