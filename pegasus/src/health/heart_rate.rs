use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyHeartRateSample {
    pub timestamp_unix: u64,
    pub bpm: u8,
}

#[derive(Debug, Clone)]
pub struct DailyHeartRateSummary {
    pub sample_count: usize,
    pub average_bpm: u8,
    pub min_bpm: u8,
    pub max_bpm: u8,
}

pub fn summarize_daily_heart_rate(samples: &[DailyHeartRateSample]) -> DailyHeartRateSummary {
    if samples.is_empty() {
        return DailyHeartRateSummary {
            sample_count: 0,
            average_bpm: 0,
            min_bpm: 0,
            max_bpm: 0,
        };
    }

    let sample_count = samples.len();

    let total: u32 = samples
        .iter()
        .map(|sample| sample.bpm as u32)
        .sum();

    let average_bpm = (total / sample_count as u32) as u8;

    let min_bpm = samples
        .iter()
        .map(|sample| sample.bpm)
        .min()
        .unwrap_or(0);

    let max_bpm = samples
        .iter()
        .map(|sample| sample.bpm)
        .max()
        .unwrap_or(0);

    DailyHeartRateSummary {
        sample_count,
        average_bpm,
        min_bpm,
        max_bpm,
    }
}
