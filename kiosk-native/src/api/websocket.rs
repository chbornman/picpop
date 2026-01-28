//! WebSocket client for real-time kiosk events.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::mpsc as tokio_mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::config;

#[derive(Error, Debug)]
pub enum WsError {
    #[error("Connection failed: {0}")]
    Connection(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("URL parse error: {0}")]
    Url(#[from] url::ParseError),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Events received from the WebSocket
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// A phone connected to the session
    PhoneConnected,
    /// A phone disconnected from the session
    PhoneDisconnected,
    /// Countdown tick (value is seconds remaining)
    Countdown(u32),
    /// A photo is ready
    PhotoReady(PhotoInfo),
    /// Capture sequence completed
    CaptureComplete,
    /// Capture failed with error message
    CaptureFailed(String),
    /// Session was ended
    SessionEnded,
    /// Connection established
    Connected,
    /// Connection lost (will attempt reconnect)
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoInfo {
    pub id: String,
    #[serde(rename = "thumbnailUrl")]
    pub thumbnail_url: String,
    #[serde(rename = "webUrl")]
    pub web_url: String,
}

#[derive(Debug, Deserialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// Handle for controlling the WebSocket connection
pub struct WsHandle {
    shutdown_tx: tokio_mpsc::Sender<()>,
}

impl WsHandle {
    /// Close the WebSocket connection
    pub async fn close(&self) {
        let _ = self.shutdown_tx.send(()).await;
    }
}

/// Callback type for WebSocket events
pub type WsCallback = Box<dyn Fn(WsEvent) + Send + Sync>;

/// Connect to the WebSocket and spawn a task to handle messages
/// Uses a callback to send events to the main thread
pub fn connect<F>(
    session_id: String,
    callback: F,
) -> WsHandle
where
    F: Fn(WsEvent) + Send + Sync + 'static,
{
    let (shutdown_tx, mut shutdown_rx) = tokio_mpsc::channel::<()>(1);
    let callback = std::sync::Arc::new(callback);

    tokio::spawn(async move {
        loop {
            let url = config::ws_url(&session_id);
            log::info!("Connecting to WebSocket: {}", url);

            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    log::info!("WebSocket connected");
                    callback(WsEvent::Connected);

                    let (mut write, mut read) = ws_stream.split();

                    loop {
                        tokio::select! {
                            _ = shutdown_rx.recv() => {
                                log::info!("WebSocket shutdown requested");
                                let _ = write.close().await;
                                return;
                            }
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        if let Some(event) = parse_message(&text) {
                                            callback(event);
                                        }
                                    }
                                    Some(Ok(Message::Ping(data))) => {
                                        let _ = write.send(Message::Pong(data)).await;
                                    }
                                    Some(Ok(Message::Close(_))) => {
                                        log::info!("WebSocket closed by server");
                                        break;
                                    }
                                    Some(Err(e)) => {
                                        log::error!("WebSocket error: {}", e);
                                        break;
                                    }
                                    None => {
                                        log::info!("WebSocket stream ended");
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to connect to WebSocket: {}", e);
                }
            }

            callback(WsEvent::Disconnected);

            // Wait before reconnecting, but check for shutdown
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    log::info!("WebSocket shutdown during reconnect wait");
                    return;
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(
                    config::WS_RECONNECT_DELAY_MS
                )) => {}
            }
        }
    });

    WsHandle { shutdown_tx }
}

fn parse_message(text: &str) -> Option<WsEvent> {
    let msg: WsMessage = match serde_json::from_str(text) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("Failed to parse WebSocket message: {} - {}", e, text);
            return None;
        }
    };

    log::debug!("Received WS message: {:?}", msg.msg_type);

    match msg.msg_type.as_str() {
        "phone_connected" => Some(WsEvent::PhoneConnected),
        "phone_disconnected" => Some(WsEvent::PhoneDisconnected),
        "countdown" => {
            let value = msg.data
                .and_then(|d| d.get("value").and_then(|v| v.as_u64()))
                .unwrap_or(0) as u32;
            Some(WsEvent::Countdown(value))
        }
        "photo_ready" => {
            msg.data.and_then(|d| {
                serde_json::from_value::<PhotoInfo>(d).ok()
            }).map(WsEvent::PhotoReady)
        }
        "capture_complete" => Some(WsEvent::CaptureComplete),
        "capture_failed" => {
            let error = msg.data
                .and_then(|d| d.get("error").and_then(|v| v.as_str()).map(String::from))
                .unwrap_or_else(|| "Unknown error".to_string());
            Some(WsEvent::CaptureFailed(error))
        }
        "session_ended" => Some(WsEvent::SessionEnded),
        _ => {
            log::warn!("Unknown message type: {}", msg.msg_type);
            None
        }
    }
}
