//! HTTP API client for PicPop backend.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::config;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("Server error: {0}")]
    Server(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub id: String,
}



/// HTTP client for the PicPop API
#[derive(Clone)]
pub struct ApiClient {
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Create a new photo session
    pub async fn create_session(&self) -> Result<CreateSessionResponse, ApiError> {
        let url = config::sessions_url();
        log::info!("Creating session at {}", url);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Server(format!("{}: {}", status, body)));
        }

        let session: CreateSessionResponse = response.json().await?;
        log::info!("Created session: {}", session.id);
        Ok(session)
    }

    /// End an active session
    pub async fn end_session(&self, session_id: &str) -> Result<(), ApiError> {
        let url = config::session_end_url(session_id);
        log::info!("Ending session {} at {}", session_id, url);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Server(format!("{}: {}", status, body)));
        }

        log::info!("Session {} ended", session_id);
        Ok(())
    }

    /// Trigger photo capture
    /// Note: This just triggers the capture - actual photo events come via WebSocket
    pub async fn capture(&self, session_id: &str) -> Result<(), ApiError> {
        let url = config::capture_url(session_id);
        log::info!("Starting capture for session {} at {}", session_id, url);

        let response = self.client.post(&url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Server(format!("{}: {}", status, body)));
        }

        // Don't parse response - just consume it
        // Photos are sent via WebSocket events
        let _ = response.bytes().await;
        log::info!("Capture request completed for session {}", session_id);
        Ok(())
    }

    /// Fetch image bytes from a URL
    pub async fn fetch_image(&self, url: &str) -> Result<Vec<u8>, ApiError> {
        log::debug!("Fetching image from {}", url);
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ApiError::Server(format!("Failed to fetch image: {}", status)));
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
