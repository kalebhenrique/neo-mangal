use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manga {
    pub id: String,
    pub title: String,
    pub url: String,
    pub cover_url: Option<String>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chapter {
    pub id: String,
    pub manga_id: String,
    pub title: String,
    pub url: String,
    pub number: Option<f32>,
}

impl Chapter {
    /// Extracts the numeric chapter number from a chapter title/string.
    /// Handles patterns like "Chapter 105", "Capítulo 45.5", "Ch. 12", "105 - Title", "Vol. 1 Ch. 12", etc.
    pub fn parse_number_from_title(title: &str) -> Option<f32> {
        let clean = crate::domain::favorite::FavoriteManager::clean_chapter_title(title);

        // 1. Keyword prefix match: Chapter, Capítulo, Capitulo, Ch., Cap., Ep., Episode, #
        if let Ok(re_kw) = Regex::new(
            r"(?i)(?:chapter|cap[ií]tulo|ch\b|ch\.|cap\b|cap\.|ep\b|ep\.|episode|epis[oó]dio|#)\s*[:.]?\s*(\d+(?:\.\d+)?)",
        ) {
            let mut last_match = None;
            for cap in re_kw.captures_iter(&clean) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f32>() {
                        last_match = Some(val);
                    }
                }
            }
            if let Some(val) = last_match {
                return Some(val);
            }
        }

        // 2. Leading number match: "105 - Title", "105: Title", or standalone "105"
        if let Ok(re_lead) = Regex::new(r"^\s*(\d+(?:\.\d+)?)(?:\s*[-:]|\s*$|\s+)") {
            if let Some(cap) = re_lead.captures(&clean) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f32>() {
                        return Some(val);
                    }
                }
            }
        }

        // 3. Fallback: Last standalone number
        if let Ok(re_num) = Regex::new(r"(?:^|\s)(\d+(?:\.\d+)?)(?:\s|$)") {
            let mut last_val = None;
            for cap in re_num.captures_iter(&clean) {
                if let Some(m) = cap.get(1) {
                    if let Ok(val) = m.as_str().parse::<f32>() {
                        last_val = Some(val);
                    }
                }
            }
            if let Some(val) = last_val {
                return Some(val);
            }
        }

        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Page {
    pub index: usize,
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chapter_numbers() {
        assert_eq!(Chapter::parse_number_from_title("Chapter 105"), Some(105.0));
        assert_eq!(Chapter::parse_number_from_title("Chapter 105.5"), Some(105.5));
        assert_eq!(Chapter::parse_number_from_title("Chapter 105: The Battle"), Some(105.0));
        assert_eq!(Chapter::parse_number_from_title("Capítulo 42"), Some(42.0));
        assert_eq!(Chapter::parse_number_from_title("Capitulo 42.1"), Some(42.1));
        assert_eq!(Chapter::parse_number_from_title("Cap. 12"), Some(12.0));
        assert_eq!(Chapter::parse_number_from_title("Ch. 77"), Some(77.0));
        assert_eq!(Chapter::parse_number_from_title("Ch.77"), Some(77.0));
        assert_eq!(Chapter::parse_number_from_title("Vol. 1 Ch. 15"), Some(15.0));
        assert_eq!(Chapter::parse_number_from_title("#99"), Some(99.0));
        assert_eq!(Chapter::parse_number_from_title("Episode 4"), Some(4.0));
        assert_eq!(Chapter::parse_number_from_title("Ep. 4"), Some(4.0));
        assert_eq!(Chapter::parse_number_from_title("105 - The Finale"), Some(105.0));
        assert_eq!(Chapter::parse_number_from_title("105: The Finale"), Some(105.0));
        assert_eq!(Chapter::parse_number_from_title("105"), Some(105.0));
        assert_eq!(Chapter::parse_number_from_title("One Piece 1050"), Some(1050.0));
    }
}
