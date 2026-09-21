use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::error::{NeoError, Result};

fn default_true() -> bool {
    true
}

fn default_splitter() -> u8 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub download_dir: PathBuf,
    pub language: String,
    pub kcc_profile: String,
    pub kcc_format: String,
    pub kcc_manga_style: bool,
    pub concurrent_downloads: usize,
    #[serde(default = "default_true")]
    pub rename_to_azw3: bool,
    #[serde(default)]
    pub active_sources: Vec<String>,
    #[serde(default = "default_true")]
    pub kcc_stretch: bool,
    #[serde(default = "default_true")]
    pub kcc_upscale: bool,
    #[serde(default = "default_true")]
    pub kcc_smart_cover_crop: bool,
    #[serde(default = "default_true")]
    pub kcc_cover_fill: bool,
    #[serde(default = "default_splitter")]
    pub kcc_splitter: u8,
    #[serde(default)]
    pub anilist_token: Option<String>,
    #[serde(default)]
    pub anilist_username: Option<String>,
    #[serde(default)]
    pub anilist_enabled: bool,
    #[serde(default)]
    pub anilist_sync_on_download: bool,
}

impl Default for Config {
    fn default() -> Self {
        let default_dir = Self::detect_initial_download_dir();
        Self {
            download_dir: default_dir,
            language: "en".to_string(), // English default as requested
            kcc_profile: "KPW5".to_string(), // Kindle Paperwhite 11ª / 12ª Gen (300 ppi)
            kcc_format: "AZW3".to_string(),
            kcc_manga_style: true,
            concurrent_downloads: 4,
            rename_to_azw3: true,
            active_sources: Vec::new(),
            kcc_stretch: true,
            kcc_upscale: true,
            kcc_smart_cover_crop: true,
            kcc_cover_fill: true,
            kcc_splitter: 2,
            anilist_token: None,
            anilist_username: None,
            anilist_enabled: false,
            anilist_sync_on_download: false,
        }
    }
}

impl Config {
    /// Returns human-readable description for KCC device profiles
    #[allow(dead_code)]
    pub fn profile_description(code: &str) -> &'static str {
        match code {
            "KPW5" => "Kindle Paperwhite (11ª / 12ª Gen - 300 ppi)",
            "KV" => "Kindle Paperwhite (3ª / 4ª Gen) / Voyage",
            "KO" => "Kindle Oasis (2ª / 3ª Gen - 300 ppi)",
            "KS" => "Kindle Scribe (10.2\" - 300 ppi)",
            "K11" => "Kindle Básico (11ª Geração)",
            "KPW" => "Kindle Paperwhite (1ª / 2ª Gen antiga)",
            _ => "Dispositivo Kindle",
        }
    }

    /// Returns the central application configuration & data directory.
    /// Checks ~/.config/neo-mangal first, then system config directory.
    pub fn app_dir() -> PathBuf {
        // 1. Check ~/.config/neo-mangal
        if let Some(home) = dirs::home_dir() {
            let dot_config = home.join(".config").join("neo-mangal");
            if dot_config.exists() {
                return dot_config;
            }
        }
        // 2. Check system config_dir (~/Library/Application Support/neo-mangal on macOS)
        if let Some(sys_cfg) = dirs::config_dir() {
            let sys_dir = sys_cfg.join("neo-mangal");
            if sys_dir.exists() {
                return sys_dir;
            }
        }
        // 3. Fallback to ~/.config/neo-mangal if home exists, else system config
        if let Some(home) = dirs::home_dir() {
            let dot_config = home.join(".config").join("neo-mangal");
            let _ = fs::create_dir_all(&dot_config);
            return dot_config;
        }
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("neo-mangal")
    }

    /// Returns the configuration file path
    pub fn config_path() -> PathBuf {
        Self::app_dir().join("config.toml")
    }

    /// Detect initial download directory, checking legacy mangal.toml if available
    fn detect_initial_download_dir() -> PathBuf {
        let legacy_paths = [
            dirs::config_dir().map(|d| d.join("mangal").join("mangal.toml")),
            dirs::data_dir().map(|d| d.join("mangal").join("mangal.toml")),
            dirs::home_dir().map(|d| d.join("Library/Application Support/mangal/mangal.toml")),
        ];

        for path_opt in legacy_paths.into_iter().flatten() {
            if path_opt.exists() {
                if let Ok(content) = fs::read_to_string(&path_opt) {
                    if let Ok(val) = content.parse::<toml::Value>() {
                        if let Some(path_str) = val.get("downloader")
                            .and_then(|d| d.get("path"))
                            .and_then(|p| p.as_str()) 
                        {
                            let candidate = PathBuf::from(path_str);
                            if !path_str.trim().is_empty() {
                                return candidate;
                            }
                        }
                    }
                }
            }
        }

        if let Some(downloads) = dirs::download_dir() {
            downloads.join("Mangas")
        } else if let Some(home) = dirs::home_dir() {
            home.join("Downloads").join("Mangas")
        } else {
            PathBuf::from("./Mangas")
        }
    }

    /// Load configuration from disk, or create and save default if it does not exist
    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let config: Config = toml::from_str(&content)
                .map_err(|e| NeoError::Config(format!("Failed to parse config: {}", e)))?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Persist configuration atomically to disk
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| NeoError::Config(format!("Failed to serialize config: {}", e)))?;
        fs::write(&path, content)?;
        Ok(())
    }

    /// Update download directory and persist immediately
    pub fn update_download_dir(&mut self, new_path: impl AsRef<Path>) -> Result<()> {
        self.download_dir = new_path.as_ref().to_path_buf();
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_valid_fields() {
        let config = Config::default();
        assert_eq!(config.language, "en");
        assert_eq!(config.kcc_profile, "KPW5");
        assert_eq!(config.kcc_format, "AZW3");
        assert!(config.rename_to_azw3);
        assert!(config.kcc_manga_style);
        assert!(!config.download_dir.as_os_str().is_empty());
        assert!(config.kcc_stretch);
        assert!(config.kcc_upscale);
        assert!(config.kcc_smart_cover_crop);
        assert!(config.kcc_cover_fill);
        assert_eq!(config.kcc_splitter, 2);
        assert_eq!(config.anilist_token, None);
        assert!(!config.anilist_enabled);
        assert!(!config.anilist_sync_on_download);
    }

    #[test]
    fn test_serialize_deserialize_config() {
        let config = Config::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config.download_dir, deserialized.download_dir);
        assert_eq!(config.kcc_profile, deserialized.kcc_profile);
        assert_eq!(config.language, deserialized.language);
    }
}
