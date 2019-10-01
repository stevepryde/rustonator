use serde_json;
use std::{error::Error, fmt};

#[derive(Debug)]
pub enum JsonError {
    SerdeError(serde_json::error::Error),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JsonError::SerdeError(x) => write!(f, "JSON Conversion Error: {}", x.to_string()),
        }
    }
}

impl Error for JsonError {
}

impl From<serde_json::error::Error> for JsonError {
    fn from(value: serde_json::error::Error) -> Self {
        JsonError::SerdeError(value)
    }
}

pub trait ToJson {
    fn to_json(&self) -> Result<serde_json::Value, JsonError>;
}
