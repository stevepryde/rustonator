use async_std::io;
use std::fmt;

pub type ZResult<T> = Result<T, ZError>;

#[derive(Debug)]
pub enum ZError {
    FatalError(String),
    IOError(String),
    WebSocketError(String),
    JsonError(String),
}

impl fmt::Display for ZError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<io::Error> for ZError {
    fn from(e: io::Error) -> Self {
        ZError::IOError(e.to_string())
    }
}

impl From<tungstenite::error::Error> for ZError {
    fn from(e: tungstenite::error::Error) -> Self {
        ZError::WebSocketError(e.to_string())
    }
}

impl From<futures::channel::mpsc::SendError> for ZError {
    fn from(e: futures::channel::mpsc::SendError) -> Self {
        ZError::WebSocketError(e.to_string())
    }
}

impl From<serde_json::error::Error> for ZError {
    fn from(e: serde_json::error::Error) -> Self {
        ZError::JsonError(e.to_string())
    }
}
