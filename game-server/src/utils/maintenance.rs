use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::sync::{Mutex, OnceLock};

const DEFAULT_MAINTENANCE_MESSAGE: &str = "Under maintenance. Please try again later.";
const MAINTENANCE_ENABLED: bool = false;

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
        #[cfg(test)]
        {
            if let Some(state) = TEST_OVERRIDE
                .get_or_init(|| Mutex::new(None))
                .lock()
                .expect("maintenance override lock poisoned")
                .clone()
            {
                return state;
            }
        }

        Self {
            enabled: MAINTENANCE_ENABLED,
            message: default_message(),
        }
    }
}

#[cfg(test)]
static TEST_OVERRIDE: OnceLock<Mutex<Option<MaintenanceState>>> = OnceLock::new();

#[cfg(test)]
pub fn set_test_override(state: Option<MaintenanceState>) {
    *TEST_OVERRIDE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .expect("maintenance override lock poisoned") = state;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_not_under_maintenance() {
        set_test_override(None);
        let state = MaintenanceState::load_current();

        assert!(!state.enabled);
        assert_eq!(state.message, DEFAULT_MAINTENANCE_MESSAGE);
    }
}
