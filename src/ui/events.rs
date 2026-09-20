use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;
use crate::domain::job::JobStatus;
use crate::domain::manga::{Chapter, Manga};
use crate::error::Result;

#[derive(Debug)]
pub enum AppEvent {
    Key(KeyEvent),
    Paste(String),
    Tick,
    // Background task events
    SearchResults(Result<Vec<Manga>>),
    ChaptersResults(Result<Vec<Chapter>>),
    DownloadStatus(JobStatus),
    KccStatus(String),
    OperationSuccess(String),
    OperationError(String),
    ChapterDownloaded {
        manga_url: String,
        chapter_title: String,
    },
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
    tx: mpsc::UnboundedSender<AppEvent>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let event_tx = tx.clone();

        // Spawn crossterm reader task
        tokio::spawn(async move {
            loop {
                if event::poll(tick_rate).unwrap_or(false) {
                    match event::read() {
                        Ok(CrosstermEvent::Key(key)) => {
                            let _ = event_tx.send(AppEvent::Key(key));
                        }
                        Ok(CrosstermEvent::Paste(text)) => {
                            let _ = event_tx.send(AppEvent::Paste(text));
                        }
                        _ => {}
                    }
                } else {
                    let _ = event_tx.send(AppEvent::Tick);
                }
            }
        });

        Self { rx, tx }
    }

    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.tx.clone()
    }

    pub async fn next(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
