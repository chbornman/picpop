//! Configuration constants for the PicPop kiosk.

/// Base URL for HTTP API calls
pub const API_BASE: &str = "http://localhost:8000";

/// Base URL for WebSocket connections
pub const WS_BASE: &str = "ws://localhost:8000";

/// Camera preview endpoint
pub const CAMERA_PREVIEW_URL: &str = "http://localhost:8000/api/v1/camera/preview";

/// QR code size in pixels (small, for collapsed view)
/// Must be at least ~150px for reliable scanning of version 6 QR codes
pub const QR_SIZE_SMALL: u32 = 150;
/// QR code size in pixels (large, for expanded view)
pub const QR_SIZE_LARGE: u32 = 280;

/// WebSocket reconnection delay in milliseconds
pub const WS_RECONNECT_DELAY_MS: u64 = 2000;

/// Error message display duration in milliseconds
pub const ERROR_DISPLAY_DURATION_MS: u64 = 5000;

/// Build the sessions API URL
pub fn sessions_url() -> String {
    format!("{}/api/v1/sessions", API_BASE)
}

/// Build the session end URL
pub fn session_end_url(session_id: &str) -> String {
    format!("{}/api/v1/sessions/{}/end", API_BASE, session_id)
}

/// Build the capture URL
pub fn capture_url(session_id: &str) -> String {
    format!("{}/api/v1/sessions/{}/capture", API_BASE, session_id)
}

/// Build the WiFi QR URL
pub fn wifi_qr_url(size: u32) -> String {
    format!("{}/api/v1/sessions/wifi-qr?size={}", API_BASE, size)
}

/// Build the session QR URL
pub fn session_qr_url(session_id: &str, size: u32) -> String {
    format!(
        "{}/api/v1/sessions/{}/qr?size={}",
        API_BASE, session_id, size
    )
}

/// Build the WebSocket URL for a session
pub fn ws_url(session_id: &str) -> String {
    format!("{}/api/v1/ws/kiosk/{}", WS_BASE, session_id)
}

/// Build full URL for a photo path
pub fn photo_url(path: &str) -> String {
    if path.starts_with("http") {
        path.to_string()
    } else {
        format!("{}{}", API_BASE, path)
    }
}
