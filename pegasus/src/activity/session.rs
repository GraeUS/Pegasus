use super::types::ActivityType;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub activity_type: ActivityType,
    pub started_at_unix: u64,
    pub ended_at_unix: Option<u64>,
    pub duration_seconds: u64,

    pub steps: u32,
    pub distance_meters: f64,
    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,
    pub notes: String,
}

impl ActivitySession {
    pub fn start(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            started_at_unix: current_unix_seconds(),
            ended_at_unix: None,
            duration_seconds: 0,

            steps: 0,
            distance_meters: 0.0,
            avg_heart_rate: 0,
            max_heart_rate: 0,
            notes: String::new(),
        }
    }

    pub fn stop(mut self) -> Self {
        let ended_at_unix = current_unix_seconds();

        self.duration_seconds = ended_at_unix.saturating_sub(self.started_at_unix);
        self.ended_at_unix = Some(ended_at_unix);

        self
    }

    pub fn elapsed(&self) -> Duration {
        if let Some(_) = self.ended_at_unix {
            Duration::from_secs(self.duration_seconds)
        } else {
            let now = current_unix_seconds();
            Duration::from_secs(now.saturating_sub(self.started_at_unix))
        }
    }
}

fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

#[derive(Debug, Clone)]
pub enum ActivityState {
    Idle,
    Selected(ActivityType),
    Active(ActivitySession),
    Completed(ActivitySession),
}

impl Default for ActivityState {
    fn default() -> Self {
        Self::Idle
    }
}
