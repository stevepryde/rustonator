use async_std::io;

pub type ZResult<T> = Result<T, ZError>;

#[derive(Debug)]
pub enum ZError {
    FatalError(String),
    IOError(String),
    WebSocketError(String),
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
