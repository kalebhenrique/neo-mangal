use futures::stream::{self, StreamExt};
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use crate::domain::job::JobStatus;
use crate::domain::manga::Page;
use crate::error::{NeoError, Result};

pub struct ChapterDownloader {
    client: reqwest::Client,
    concurrency: usize,
}

impl ChapterDownloader {
    pub fn new(concurrency: usize) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        Self {
            client,
            concurrency: concurrency.max(1),
        }
    }

    /// Sanitizes directory or file names for filesystem safety
    pub fn sanitize_name(name: &str) -> String {
        let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\0'];
        let mut clean = name
            .chars()
            .map(|c| if invalid_chars.contains(&c) { '_' } else { c })
            .collect::<String>()
            .trim()
            .to_string();

        if clean.is_empty() {
            clean = "untitled".to_string();
        }
        clean
    }

    /// Downloads all pages in parallel with bounded concurrency and progress reporting
    pub async fn download_chapter(
        &self,
        pages: &[Page],
        target_dir: &Path,
        progress_tx: Option<mpsc::Sender<JobStatus>>,
    ) -> Result<PathBuf> {
        fs::create_dir_all(target_dir)?;

        let total = pages.len();
        if total == 0 {
            return Err(NeoError::Download("Chapter has no pages to download".to_string()));
        }

        let completed = Arc::new(AtomicUsize::new(0));
        let pages_vec = pages.to_vec();
        let client = self.client.clone();
        let target_dir_buf = target_dir.to_path_buf();

        let download_stream = stream::iter(pages_vec).map(move |page| {
            let client = client.clone();
            let page_url = page.url.clone();
            let page_index = page.index;
            let target_path = target_dir_buf.join(format!("{:04}.png", page_index));
            let completed = Arc::clone(&completed);
            let tx = progress_tx.clone();

            async move {
                // If file already exists and has non-zero size, skip re-download
                if target_path.exists() && fs::metadata(&target_path).map(|m| m.len() > 0).unwrap_or(false) {
                    let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
                    if let Some(ref sender) = tx {
                        let _ = sender.send(JobStatus::Downloading { current, total }).await;
                    }
                    return Ok::<(), NeoError>(());
                }

                let mut req = client.get(&page_url);
                if let Ok(parsed) = reqwest::Url::parse(&page_url) {
                    if let Some(host) = parsed.host_str() {
                        req = req.header("Referer", format!("https://{}/", host));
                    }
                }
                let response = req.send().await?;

                if !response.status().is_success() {
                    return Err(NeoError::Download(format!(
                        "HTTP {} downloading page {}",
                        response.status(),
                        page_index
                    )));
                }

                let bytes = response.bytes().await?;
                let mut file = File::create(&target_path)?;
                file.write_all(&bytes)?;

                let current = completed.fetch_add(1, Ordering::SeqCst) + 1;
                if let Some(ref sender) = tx {
                    let _ = sender.send(JobStatus::Downloading { current, total }).await;
                }

                Ok(())
            }
        });

        // Run bounded concurrent downloads
        let results: Vec<Result<()>> = download_stream
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        for res in results {
            res?;
        }

        Ok(target_dir.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(ChapterDownloader::sanitize_name("One / Piece: Void?"), "One _ Piece_ Void_");
        assert_eq!(ChapterDownloader::sanitize_name(""), "untitled");
    }
}
