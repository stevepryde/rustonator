use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

const DEFAULT_MAINTENANCE_MESSAGE: &str = "Under maintenance. Please try again later.";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MaintenanceState {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_message")]
    pub message: String,
}

fn default_message() -> String {
    DEFAULT_MAINTENANCE_MESSAGE.to_string()
}

impl Default for MaintenanceState {
    fn default() -> Self {
        Self {
            enabled: false,
            message: default_message(),
        }
    }
}

impl MaintenanceState {
    pub fn load_current() -> Self {
        let path = std::env::var("MAINTENANCE_FILE")
            .unwrap_or_else(|_| "maintenance.json".to_string());
        Self::load_from_path(Path::new(&path))
    }

    pub fn load_from_path(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_defaults_to_not_under_maintenance() {
        let path = Path::new("/definitely/missing/maintenance.json");
        let state = MaintenanceState::load_from_path(path);

        assert!(!state.enabled);
        assert_eq!(state.message, DEFAULT_MAINTENANCE_MESSAGE);
    }
}
