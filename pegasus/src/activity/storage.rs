use super::session::ActivitySession;
use std::{
    fs,
    io,
    path::PathBuf,
};

fn activity_history_path() -> io::Result<PathBuf> {
    let mut data_dir = dirs::data_local_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not find local data directory"))?;

    data_dir.push("pegasus");
    fs::create_dir_all(&data_dir)?;

    data_dir.push("activity_history.json");

    Ok(data_dir)
}

pub fn load_activity_history() -> io::Result<Vec<ActivitySession>> {
    let path = activity_history_path()?;

    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)?;

    let sessions = serde_json::from_str::<Vec<ActivitySession>>(&contents)
        .unwrap_or_else(|_| Vec::new());

    Ok(sessions)
}

pub fn save_activity_session(session: &ActivitySession) -> io::Result<()> {
    let mut sessions = load_activity_history()?;
    sessions.push(session.clone());

    let path = activity_history_path()?;
    let contents = serde_json::to_string_pretty(&sessions)?;

    fs::write(path, contents)?;

    Ok(())
}
