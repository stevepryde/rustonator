use crate::comms::websocket::WsError;

pub type ZResult<T> = Result<T, ZError>;

#[derive(Debug, thiserror::Error)]
pub enum ZError {
    #[error("Fatal error: {0}")]
    FatalError(String),
    #[error("IO error: {0}")]
    IOError(String),
    #[error("WebSocket error: {0:?}")]
    WebSocketError(WsError),
    #[error("JSON error: {0}")]
    JsonError(String),
}

impl From<serde_json::error::Error> for ZError {
    fn from(e: serde_json::error::Error) -> Self {
        ZError::JsonError(e.to_string())
    }
}

impl From<WsError> for ZError {
    fn from(e: WsError) -> Self {
        ZError::WebSocketError(e)
    }
}
