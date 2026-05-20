use super::heart_rate::DailyHeartRateSample;
use std::{
    fs,
    io,
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

fn health_data_dir() -> io::Result<PathBuf> {
    let mut data_dir = dirs::data_local_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find local data directory"))?;

    data_dir.push("pegasus");
    data_dir.push("health");

    fs::create_dir_all(&data_dir)?;

    Ok(data_dir)
}

fn daily_heart_rate_path() -> io::Result<PathBuf> {
    let mut path = health_data_dir()?;
    path.push("daily_heart_rate.json");
    Ok(path)
}

pub fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

pub fn load_daily_heart_rate_samples() -> io::Result<Vec<DailyHeartRateSample>> {
    let path = daily_heart_rate_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;

    let samples = serde_json::from_str::<Vec<DailyHeartRateSample>>(&contents)
        .unwrap_or_else(|_| Vec::new());

    Ok(samples)
}

pub fn save_daily_heart_rate_samples(samples: &[DailyHeartRateSample]) -> io::Result<()> {
    let path = daily_heart_rate_path()?;
    let contents = serde_json::to_string_pretty(samples)?;

    fs::write(path, contents)?;

    Ok(())
}

pub fn add_daily_heart_rate_sample(bpm: u8) -> io::Result<DailyHeartRateSample> {
    let mut samples = load_daily_heart_rate_samples()?;

    let sample = DailyHeartRateSample {
        timestamp_unix: current_unix_seconds(),
        bpm,
    };

    samples.push(sample.clone());
    save_daily_heart_rate_samples(&samples)?;

    Ok(sample)
}
