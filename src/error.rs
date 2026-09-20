use thiserror::Error;

#[derive(Error, Debug)]
pub enum NeoError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("HTML parsing error: {0}")]
    Parse(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("CBZ archive error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("Toolchain missing: {0}")]
    ToolchainMissing(String),

    #[error("KCC conversion failed: {0}")]
    Kcc(String),

    #[error("Download error: {0}")]
    Download(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, NeoError>;
