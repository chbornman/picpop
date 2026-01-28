//! API clients for PicPop backend communication.

pub mod http;
pub mod websocket;

pub use http::ApiClient;
pub use websocket::{WsEvent, WsHandle, PhotoInfo};
