use crate::comms::websocket::WsError;
use std::fmt;

pub type ZResult<T> = Result<T, ZError>;

#[derive(Debug)]
pub enum ZError {
    FatalError(String),
    IOError(String),
    WebSocketError(WsError),
    JsonError(String),
}

impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
