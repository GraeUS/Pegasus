use super::types::ActivityType;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub activity_type: ActivityType,
    pub started_at_unix: u64,
    pub ended_at_unix: Option<u64>,
    pub duration_seconds: u64,
    pub heart_rate_samples: Vec<ActivityHeartRateSample>,

    pub steps: u32,
    pub distance_meters: f64,
    pub avg_heart_rate: u8,
    pub max_heart_rate: u8,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityHeartRateSample {
    pub timestamp_offset_seconds: u64,
    pub bpm: u8,
}

impl ActivitySession {
    pub fn start(activity_type: ActivityType) -> Self {
        Self {
            activity_type,
            started_at_unix: current_unix_seconds(),
            ended_at_unix: None,
            duration_seconds: 0,
            heart_rate_samples: Vec::new(),

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

    pub fn add_heart_rate_sample(&mut self, bpm: u8) {
        let timestamp_offset_seconds = self.elapsed().as_secs();

        self.heart_rate_samples.push(ActivityHeartRateSample {
            timestamp_offset_seconds,
            bpm,
        });

        self.max_heart_rate = self
            .heart_rate_samples
            .iter()
            .map(|sample| sample.bpm)
            .max()
            .unwrap_or(0);

        let total: u32 = self
            .heart_rate_samples
            .iter()
            .map(|sample| sample.bpm as u32)
            .sum();

        let count = self.heart_rate_samples.len() as u32;

        self.avg_heart_rate = if count > 0 {
            (total / count) as u8
        } else {
            0
        };
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
    ViewingHistory(usize),
}

impl Default for ActivityState {
    fn default() -> Self {
        Self::Idle
    }
}
