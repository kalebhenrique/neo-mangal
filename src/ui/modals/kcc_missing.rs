use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::converter::toolchain::ToolchainStatus;
use crate::i18n::AppLanguage;
use crate::ui::modals::settings::centered_rect;

pub struct KccMissingModal {
    pub status: ToolchainStatus,
}

impl KccMissingModal {
    pub fn new(status: ToolchainStatus) -> Self {
        Self { status }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, lang: AppLanguage) {
        let modal_area = centered_rect(80, 65, area);
        frame.render_widget(Clear, modal_area);

        let title = match lang {
            AppLanguage::English => " 󰀪 Missing KCC Dependencies ",
            AppLanguage::Portuguese => " 󰀪 Dependências KCC Ausentes ",
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3), // Warning header
                Constraint::Min(6),    // Install steps
                Constraint::Length(2), // Close shortcut
            ])
            .split(modal_area);

        frame.render_widget(block, modal_area);

        // Header message
        let header_text = match lang {
            AppLanguage::English => "Automated Kindle conversion requires tools not found in PATH:",
            AppLanguage::Portuguese => "O fluxo de conversão automática do KCC requer ferramentas que não foram localizadas no PATH:",
        };
        let missing_label = match lang {
            AppLanguage::English => "Missing tools: ",
            AppLanguage::Portuguese => "Ferramentas em falta: ",
        };

        let header = Paragraph::new(vec![
            Line::from(vec![
                Span::styled(header_text, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::raw(missing_label),
                Span::styled(
                    self.status.missing_tools().join(", "),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            ]),
        ]);
        frame.render_widget(header, chunks[0]);

        // Instructions block
        let mut lines = Vec::new();
        let subtitle = match lang {
            AppLanguage::English => "Installation instructions for your system:",
            AppLanguage::Portuguese => "Instruções de instalação para o seu sistema:",
        };
        lines.push(Line::from(Span::styled(subtitle, Style::default().fg(Color::White).add_modifier(Modifier::UNDERLINED))));
        lines.push(Line::from(""));

        let instructions_text = self.status.install_instructions();
        for item in instructions_text.split("\n\n") {
            for sub in item.lines() {
                if sub.starts_with("•") {
                    lines.push(Line::from(Span::styled(sub.to_string(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))));
                } else {
                    lines.push(Line::from(Span::styled(format!("  {}", sub.trim()), Style::default().fg(Color::Green))));
                }
            }
            lines.push(Line::from(""));
        }

        let instructions = Paragraph::new(lines);
        frame.render_widget(instructions, chunks[1]);

        // Footer dismiss
        let footer_label = match lang {
            AppLanguage::English => "Close this warning and return to list",
            AppLanguage::Portuguese => "Fechar este aviso e voltar para a lista",
        };
        let footer = Paragraph::new(Line::from(vec![
            Span::styled("[Enter] / [Esc] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::raw(footer_label),
        ])).alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }
}
