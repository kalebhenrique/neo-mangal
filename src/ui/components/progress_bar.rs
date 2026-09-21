use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Gauge},
    Frame,
};
use crate::domain::job::JobStatus;
use crate::i18n::{AppLanguage, I18n};

pub struct ProgressBarComponent;

impl ProgressBarComponent {
    pub fn render(frame: &mut Frame, area: Rect, status: &JobStatus, lang: AppLanguage) {
        let (ratio, label, color) = match status {
            JobStatus::Idle => (0.0, I18n::progress_ready(lang).to_string(), Color::DarkGray),
            JobStatus::Downloading { current, total } => {
                let r = if *total > 0 {
                    (*current as f64) / (*total as f64)
                } else {
                    0.0
                };
                let pct = (r * 100.0) as u16;
                (r.clamp(0.0, 1.0), I18n::progress_downloading(lang, *current, *total, pct), Color::Cyan)
            }
            JobStatus::PackagingCbz => (0.75, I18n::progress_packaging(lang).to_string(), Color::Yellow),
            JobStatus::PackagingPdf => (0.75, I18n::progress_packaging_pdf(lang).to_string(), Color::Yellow),
            JobStatus::ConvertingKcc { message } => (0.90, I18n::progress_converting(lang, message), Color::Magenta),
            JobStatus::Done(msg) => (1.0, I18n::progress_done(lang, msg), Color::Green),
            JobStatus::Failed(err) => (1.0, I18n::progress_error(lang, err), Color::Red),
        };

        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title(I18n::progress_title(lang)))
            .gauge_style(
                Style::default()
                    .fg(color)
                    .bg(Color::Rgb(30, 30, 30))
                    .add_modifier(Modifier::BOLD),
            )
            .ratio(ratio)
            .label(label);

        frame.render_widget(gauge, area);
    }
}
