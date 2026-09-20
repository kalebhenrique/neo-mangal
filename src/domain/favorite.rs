use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use crate::domain::manga::Manga;
use crate::error::{NeoError, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FavoriteManga {
    pub url: String,
    pub title: String,
    pub provider: String,
    pub cover_url: Option<String>,
    pub last_downloaded_chapter: Option<String>,
    pub last_downloaded_at: Option<String>,
}

impl FavoriteManga {
    pub fn from_manga(manga: &Manga) -> Self {
        Self {
            url: manga.url.clone(),
            title: manga.title.clone(),
            provider: manga.provider.clone(),
            cover_url: manga.cover_url.clone(),
            last_downloaded_chapter: None,
            last_downloaded_at: None,
        }
    }

    pub fn to_manga(&self) -> Manga {
        Manga {
            id: self.url.clone(),
            title: self.title.clone(),
            url: self.url.clone(),
            cover_url: self.cover_url.clone(),
            provider: self.provider.clone(),
        }
    }
}

pub struct FavoriteManager;

impl FavoriteManager {
    /// Returns path to favorites.json file in config directory
    pub fn favorites_file() -> PathBuf {
        let base = crate::config::Config::app_dir();
        if !base.exists() {
            let _ = fs::create_dir_all(&base);
        }
        base.join("favorites.json")
    }

    /// Cleans chapter title from scraper artifacts like "Last Read" and ISO timestamps
    pub fn clean_chapter_title(title: &str) -> String {
        let mut clean = title.to_string();
        if let Some(idx) = clean.find("Last Read") {
            clean = clean[..idx].trim().to_string();
        }
        if let Some(idx) = clean.find("Last read") {
            clean = clean[..idx].trim().to_string();
        }
        if let Some(idx) = clean.find("last read") {
            clean = clean[..idx].trim().to_string();
        }
        let words: Vec<&str> = clean.split_whitespace().collect();
        let filtered: Vec<&str> = words
            .into_iter()
            .filter(|w| {
                !(w.len() >= 10 && w.chars().take(4).all(|c| c.is_ascii_digit()) && w.contains('-'))
            })
            .collect();
        filtered.join(" ")
    }

    /// Loads all favorite mangas from disk
    pub fn load_favorites() -> Vec<FavoriteManga> {
        let file = Self::favorites_file();
        if !file.exists() {
            return Vec::new();
        }

        if let Ok(content) = fs::read_to_string(&file) {
            let mut list: Vec<FavoriteManga> = serde_json::from_str(&content).unwrap_or_default();
            for fav in &mut list {
                if let Some(ref ch) = fav.last_downloaded_chapter {
                    fav.last_downloaded_chapter = Some(Self::clean_chapter_title(ch));
                }
            }
            list
        } else {
            Vec::new()
        }
    }

    /// Saves the favorite mangas list to disk
    pub fn save_favorites(favorites: &[FavoriteManga]) -> Result<()> {
        let file = Self::favorites_file();
        if let Some(parent) = file.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(favorites)
            .map_err(|e| NeoError::Other(format!("Failed to serialize favorites: {}", e)))?;
        fs::write(&file, json)?;
        Ok(())
    }

    /// Checks if a manga is currently in the favorites list by URL
    pub fn is_favorite(favorites: &[FavoriteManga], url: &str) -> bool {
        favorites.iter().any(|f| f.url == url)
    }

    /// Toggles favorite status for a manga: adds if missing, removes if present.
    /// Returns true if added, false if removed.
    pub fn toggle_favorite(favorites: &mut Vec<FavoriteManga>, manga: &Manga) -> bool {
        if let Some(pos) = favorites.iter().position(|f| f.url == manga.url) {
            favorites.remove(pos);
            let _ = Self::save_favorites(favorites);
            false
        } else {
            favorites.push(FavoriteManga::from_manga(manga));
            let _ = Self::save_favorites(favorites);
            true
        }
    }

    /// Removes a manga from the favorites list by URL.
    /// Returns true if removed, false if it was not in favorites.
    pub fn remove_favorite(favorites: &mut Vec<FavoriteManga>, url: &str) -> bool {
        if let Some(pos) = favorites.iter().position(|f| f.url == url) {
            favorites.remove(pos);
            let _ = Self::save_favorites(favorites);
            true
        } else {
            false
        }
    }

    /// Updates the last downloaded chapter for a manga in favorites if it exists
    pub fn update_last_downloaded(
        favorites: &mut [FavoriteManga],
        manga_url: &str,
        chapter_title: &str,
    ) -> bool {
        if let Some(fav) = favorites.iter_mut().find(|f| f.url == manga_url) {
            fav.last_downloaded_chapter = Some(Self::clean_chapter_title(chapter_title));
            fav.last_downloaded_at = Some(chrono_now_formatted());
            let _ = Self::save_favorites(favorites);
            true
        } else {
            false
        }
    }
}

fn chrono_now_formatted() -> String {
    // Standard timestamp without heavy external chrono crate
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => {
            let secs = d.as_secs();
            let days = secs / 86400;
            // Approximate calendar formatting or relative
            format!("day-{}", days)
        }
        Err(_) => "recently".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle_favorite() {
        let mut favs = Vec::new();
        let manga = Manga {
            id: "u1".into(),
            title: "One Piece".into(),
            url: "https://example.com/one-piece".into(),
            cover_url: None,
            provider: "WeebCentral".into(),
        };

        // Add
        let added = FavoriteManager::toggle_favorite(&mut favs, &manga);
        assert!(added);
        assert_eq!(favs.len(), 1);
        assert_eq!(favs[0].title, "One Piece");

        // Update last downloaded
        let updated = FavoriteManager::update_last_downloaded(&mut favs, &manga.url, "Chapter 1110");
        assert!(updated);
        assert_eq!(favs[0].last_downloaded_chapter.as_deref(), Some("Chapter 1110"));

        // Remove
        let removed = FavoriteManager::toggle_favorite(&mut favs, &manga);
        assert!(!removed);
        assert_eq!(favs.len(), 0);

        // Test explicit remove_favorite
        FavoriteManager::toggle_favorite(&mut favs, &manga);
        assert_eq!(favs.len(), 1);
        let removed_explicit = FavoriteManager::remove_favorite(&mut favs, &manga.url);
        assert!(removed_explicit);
        assert_eq!(favs.len(), 0);
        let removed_again = FavoriteManager::remove_favorite(&mut favs, &manga.url);
        assert!(!removed_again);
    }
}
