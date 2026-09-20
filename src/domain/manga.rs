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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Page {
    pub index: usize,
    pub url: String,
}
