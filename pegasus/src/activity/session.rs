use super::types::ActivityType;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub struct ActivitySession {
    pub activity_type: ActivityType,
    pub started_at: SystemTime,
    pub ended_at: Option<SystemTime>,
    pub duration: Duration,

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
        started_at: SystemTime::now(),
        ended_at: None,
        duration: Duration::from_secs(0),

        steps: 0,
        distance_meters: 0.0,
        avg_heart_rate: 0,
        max_heart_rate: 0,
        notes: String::new(),
    }
}

    pub fn stop(mut self) -> Self {
        let ended_at = SystemTime::now();

        self.duration = ended_at
            .duration_since(self.started_at)
            .unwrap_or_else(|_| Duration::from_secs(0));

        self.ended_at = Some(ended_at);
        self
    }

    pub fn elapsed(&self) -> Duration {
        match self.ended_at {
            Some(_) => self.duration,
            None => SystemTime::now()
                .duration_since(self.started_at)
                .unwrap_or_else(|_| Duration::from_secs(0)),
        }
    }
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
