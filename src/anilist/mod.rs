use serde_json::json;
use crate::error::{NeoError, Result};

pub struct AnilistClient;

impl AnilistClient {
    const GRAPHQL_URL: &'static str = "https://graphql.anilist.co";

    /// Verify an AniList access token and return the viewer's username
    pub async fn verify_token(token: &str) -> Result<String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let query = json!({
            "query": "query { Viewer { id name } }"
        });

        let resp = client
            .post(Self::GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token.trim()))
            .json(&query)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(NeoError::Other(format!("AniList HTTP error: {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        if let Some(errors) = body.get("errors") {
            let msg = errors[0]["message"].as_str().unwrap_or("Unauthorized");
            return Err(NeoError::Other(format!("AniList auth failed: {}", msg)));
        }

        let username = body["data"]["Viewer"]["name"]
            .as_str()
            .ok_or_else(|| NeoError::Other("Failed to parse AniList username".to_string()))?;

        Ok(username.to_string())
    }

    /// Search AniList for a manga title and return its media ID
    pub async fn search_manga(title: &str) -> Result<Option<i64>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let query = json!({
            "query": "query ($search: String) { Media (search: $search, type: MANGA) { id title { romaji english } chapters } }",
            "variables": {
                "search": title
            }
        });

        let resp = client
            .post(Self::GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .json(&query)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Ok(None);
        }

        let body: serde_json::Value = resp.json().await?;
        if let Some(id) = body["data"]["Media"]["id"].as_i64() {
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    /// Update reading progress on AniList for a media ID
    pub async fn update_progress(token: &str, media_id: i64, progress: i32) -> Result<()> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let mutation = json!({
            "query": "mutation ($ID: Int, $progress: Int) { SaveMediaListEntry (mediaId: $ID, progress: $progress, status: CURRENT) { id progress status } }",
            "variables": {
                "ID": media_id,
                "progress": progress
            }
        });

        let resp = client
            .post(Self::GRAPHQL_URL)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("Authorization", format!("Bearer {}", token.trim()))
            .json(&mutation)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(NeoError::Other(format!("AniList update failed: HTTP {}", resp.status())));
        }

        let body: serde_json::Value = resp.json().await?;
        if let Some(errors) = body.get("errors") {
            let msg = errors[0]["message"].as_str().unwrap_or("Failed to save progress");
            return Err(NeoError::Other(format!("AniList mutation error: {}", msg)));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verify_token_invalid_returns_error() {
        let res = AnilistClient::verify_token("invalid_token_12345").await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_search_manga_public() {
        let res = AnilistClient::search_manga("Berserk").await;
        assert!(res.is_ok());
        let id = res.unwrap();
        assert_eq!(id, Some(30002));
    }
}
