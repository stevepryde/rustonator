use chrono::{Datelike, Utc};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};
use tokio::sync::{Mutex, Notify};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: u32,
    pub timestamp: i64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct ScoreData {
    entries: Vec<ScoreEntry>,
}

#[derive(Debug, Serialize)]
pub struct ScoreBuckets {
    pub today: Vec<ScoreEntry>,
    pub month: Vec<ScoreEntry>,
    pub all_time: Vec<ScoreEntry>,
}

#[derive(Clone)]
pub struct ScoreStore {
    data: Arc<Mutex<ScoreData>>,
    save_notify: Arc<Notify>,
}

impl ScoreStore {
    pub fn new(path: PathBuf) -> Self {
        let data = match std::fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => ScoreData::default(),
        };

        let store = Self {
            data: Arc::new(Mutex::new(data)),
            save_notify: Arc::new(Notify::new()),
        };

        // Spawn background save task with debounce.
        let data_ref = store.data.clone();
        let notify = store.save_notify.clone();
        tokio::spawn(async move {
            loop {
                notify.notified().await;
                // Debounce: wait 5 seconds for more writes.
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                let data = data_ref.lock().await;
                match serde_json::to_string_pretty(&*data) {
                    Ok(json) => {
                        if let Err(e) = tokio::fs::write(&path, json).await {
                            error!("Failed to save scores: {e}");
                        } else {
                            info!("Scores saved to {}", path.display());
                        }
                    }
                    Err(e) => error!("Failed to serialize scores: {e}"),
                }
            }
        });

        store
    }

    pub async fn add(&self, name: String, score: u32) {
        let entry = ScoreEntry {
            name,
            score,
            timestamp: Utc::now().timestamp(),
        };
        self.data.lock().await.entries.push(entry);
        self.save_notify.notify_one();
    }

    pub async fn get_buckets(&self) -> ScoreBuckets {
        let data = self.data.lock().await;
        let now = Utc::now();
        let today_start = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();
        let month_start = now
            .date_naive()
            .with_day(1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();

        let mut today: Vec<ScoreEntry> = data
            .entries
            .iter()
            .filter(|e| e.timestamp >= today_start)
            .cloned()
            .collect();
        today.sort_by(|a, b| b.score.cmp(&a.score));
        today.truncate(10);

        let mut month: Vec<ScoreEntry> = data
            .entries
            .iter()
            .filter(|e| e.timestamp >= month_start)
            .cloned()
            .collect();
        month.sort_by(|a, b| b.score.cmp(&a.score));
        month.truncate(10);

        let mut all_time: Vec<ScoreEntry> = data.entries.clone();
        all_time.sort_by(|a, b| b.score.cmp(&a.score));
        all_time.truncate(10);

        ScoreBuckets {
            today,
            month,
            all_time,
        }
    }
}
