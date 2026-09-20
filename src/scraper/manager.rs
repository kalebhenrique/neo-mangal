use std::fs;
use std::path::{Path, PathBuf};
use crate::error::{NeoError, Result};
use crate::scraper::lua::LuaScraper;

pub const DEFAULT_REPO: &str = "kalebhenrique/neo-mangal-scrapers";

pub struct SourceManager;

impl SourceManager {
    /// Returns the path to the sources directory: ~/.config/neo-mangal/sources or ~/Library/Application Support/neo-mangal/sources
    pub fn sources_dir() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            let dot_config = home.join(".config").join("neo-mangal").join("sources");
            if dot_config.exists() {
                return dot_config;
            }
        }
        let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        base.join("neo-mangal").join("sources")
    }

    /// Ensures the sources directory exists
    pub fn ensure_sources_dir() -> Result<PathBuf> {
        let dir = Self::sources_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Resets all installed scrapers and synchronizes with local workspace or GitHub
    pub fn reset_sources() -> Result<Vec<String>> {
        let dir = Self::sources_dir();
        if dir.exists() {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                        let _ = fs::remove_file(path);
                    }
                }
            }
        }
        Self::install_sources_from_repo(DEFAULT_REPO)
    }

    /// Lists all installed .lua scrapers
    pub fn list_sources() -> Vec<String> {
        let dir = Self::sources_dir();
        if !dir.exists() {
            return Vec::new();
        }

        let mut sources = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        sources.push(stem.to_string());
                    }
                }
            }
        }
        sources.sort();
        sources
    }

    /// Loads a scraper by name (e.g. "WeebCentral")
    pub fn load_source(name: &str) -> Result<LuaScraper> {
        let dir = Self::sources_dir();
        let path = dir.join(format!("{}.lua", name));
        if path.exists() {
            return LuaScraper::from_file(&path);
        }

        // Check if path is given directly
        let direct_path = Path::new(name);
        if direct_path.exists() {
            return LuaScraper::from_file(direct_path);
        }

        Err(NeoError::Other(format!("Scraper source '{}' not found in {:?}", name, dir)))
    }

    /// Downloads and installs official scrapers from GitHub (kalebhenrique/neo-mangal-scrapers)
    pub fn install_sources_from_repo(repo: &str) -> Result<Vec<String>> {
        let target_dir = Self::ensure_sources_dir()?;
        let mut installed = Vec::new();

        // 0. Check local workspace clone (~/work/neo-mangal-scrapers/scrapers) for instant install
        let local_dir = dirs::home_dir()
            .map(|h| h.join("work").join("neo-mangal-scrapers").join("scrapers"))
            .filter(|p| p.exists());

        if let Some(local_path) = local_dir {
            if let Ok(entries) = fs::read_dir(local_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("lua") {
                        if let Some(filename) = path.file_name() {
                            let dest = target_dir.join(filename);
                            let _ = fs::copy(&path, &dest);
                            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                            if !stem.is_empty() {
                                installed.push(stem.to_string());
                            }
                        }
                    }
                }
            }
            if !installed.is_empty() {
                installed.sort();
                installed.dedup();
                return Ok(installed);
            }
        }

        let client = reqwest::blocking::Client::builder()
            .user_agent("neo-mangal")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| NeoError::Other(e.to_string()))?;

        // 1. Try fetching directory list via GitHub API
        let api_url = format!("https://api.github.com/repos/{}/contents/scrapers", repo);

        if let Ok(resp) = client.get(&api_url).send() {
            if resp.status().is_success() {
                if let Ok(items) = resp.json::<Vec<serde_json::Value>>() {
                    for item in items {
                        if let (Some(name), Some(download_url)) = (
                            item.get("name").and_then(|n| n.as_str()),
                            item.get("download_url").and_then(|u| u.as_str()),
                        ) {
                            if name.ends_with(".lua") {
                                if let Ok(content_resp) = client.get(download_url).send() {
                                    if let Ok(content) = content_resp.text() {
                                        let out_path = target_dir.join(name);
                                        let _ = fs::write(&out_path, content);
                                        let stem = name.strip_suffix(".lua").unwrap_or(name);
                                        installed.push(stem.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 2. If API was rate limited or empty, fallback to raw URLs for curated scrapers
        if installed.is_empty() {
            let fallback_scrapers = [
                "WeebCentral.lua",
                "MangaDex.lua",
                "MangaKakalot.lua",
                "Manganelo.lua",
                "Mangasee.lua",
                "AsuraScans.lua",
                "FlameScans.lua",
                "ComicK.lua",
            ];

            for filename in fallback_scrapers {
                let raw_url = format!(
                    "https://raw.githubusercontent.com/{}/main/scrapers/{}",
                    repo, filename
                );
                if let Ok(resp) = client.get(&raw_url).send() {
                    if resp.status().is_success() {
                        if let Ok(content) = resp.text() {
                            let out_path = target_dir.join(filename);
                            let _ = fs::write(&out_path, content);
                            let stem = filename.strip_suffix(".lua").unwrap_or(filename);
                            installed.push(stem.to_string());
                        }
                    }
                }
            }
        }

        if installed.is_empty() {
            return Err(NeoError::Other(format!(
                "Could not download scrapers from repository '{}'. Please check your internet connection.",
                repo
            )));
        }

        installed.sort();
        installed.dedup();
        Ok(installed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_dir_path() {
        let p = SourceManager::sources_dir();
        assert!(p.to_string_lossy().contains("sources"));
    }

    #[test]
    fn test_list_and_load_source() {
        let sources = SourceManager::list_sources();
        assert!(!sources.is_empty(), "Sources should have been installed");
        assert!(sources.contains(&"WeebCentral".to_string()));

        let weebcentral = SourceManager::load_source("WeebCentral");
        assert!(weebcentral.is_ok());

        // Live test Lua scraper execution
        let scraper = weebcentral.unwrap();
        let mangas = scraper.search("Naruto");
        assert!(mangas.is_ok());
        let results = mangas.unwrap();
        assert!(!results.is_empty(), "Should find at least 1 manga for Naruto");
        assert!(results[0].title.to_lowercase().contains("naruto"));

        // Live test MangaDex Lua scraper loading & query (if network available)
        let mangadex = SourceManager::load_source("MangaDex");
        assert!(mangadex.is_ok());
        let md_scraper = mangadex.unwrap();
        if let Ok(md_mangas) = md_scraper.search("One Piece") {
            assert!(!md_mangas.is_empty(), "Should find at least 1 manga for One Piece on MangaDex");
        }
    }
}
