use std::path::PathBuf;
use crate::domain::manga::Chapter;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Idle,
    Downloading { current: usize, total: usize },
    PackagingCbz,
    ConvertingKcc { message: String },
    Done(String),
    Failed(String),
}

#[allow(dead_code)]
impl JobStatus {
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            JobStatus::Downloading { .. } | JobStatus::PackagingCbz | JobStatus::ConvertingKcc { .. }
        )
    }

    pub fn message(&self) -> String {
        match self {
            JobStatus::Idle => "Idle".to_string(),
            JobStatus::Downloading { current, total } => {
                format!("Downloading pages: {} / {}", current, total)
            }
            JobStatus::PackagingCbz => "Packaging CBZ archive...".to_string(),
            JobStatus::ConvertingKcc { message } => format!("KCC: {}", message),
            JobStatus::Done(msg) => format!("Completed: {}", msg),
            JobStatus::Failed(err) => format!("Failed: {}", err),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DownloadJob {
    pub manga_title: String,
    pub chapter: Chapter,
    pub output_dir: PathBuf,
    pub custom_cover: Option<PathBuf>,
    pub convert_to_kcc: bool,
    pub status: JobStatus,
}
