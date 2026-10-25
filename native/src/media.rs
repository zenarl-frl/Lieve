use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use windows::Media::Control::*;

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    pub position: u32,
    pub duration: u32,
    pub available: bool,
}

pub enum Command { Toggle, Previous, Next, Seek(u32) }

#[cfg(test)]
mod tests {
    #[test]
    #[ignore = "requires a Windows desktop session"]
    fn windows_media_manager_is_accessible() {
        unsafe { crate::initialize_runtime(); }
        let manager = super::GlobalSystemMediaTransportControlsSessionManager::RequestAsync()
            .and_then(|op| op.get()).expect("Windows media manager");
        let sessions = manager.GetSessions().expect("media session list");
        println!("Available Windows media sessions: {}", sessions.Size().unwrap());
    }
}

pub fn start() -> (Sender<Command>, Receiver<Snapshot>) {
    let (commands, input) = mpsc::channel();
    let (output, snapshots) = mpsc::channel();
    std::thread::spawn(move || {
        unsafe { super::initialize_runtime(); }
        let Ok(manager) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().and_then(|op| op.get()) else { return; };
        loop {
            let command = match input.recv_timeout(Duration::from_secs(1)) {
                Ok(c) => Some(c),
                Err(mpsc::RecvTimeoutError::Timeout) => None,
                Err(_) => return,
            };
            let snapshot = (|| -> windows::core::Result<Snapshot> {
                let session = manager.GetCurrentSession()?;
                if let Some(command) = command {
                    match command {
                        Command::Toggle => { let _ = session.TryTogglePlayPauseAsync()?.get(); }
                        Command::Previous => { let _ = session.TrySkipPreviousAsync()?.get(); }
                        Command::Next => { let _ = session.TrySkipNextAsync()?.get(); }
                        Command::Seek(seconds) => { let _ = session.TryChangePlaybackPositionAsync(seconds as i64 * 10_000_000)?.get(); }
                    }
                }
                let properties = session.TryGetMediaPropertiesAsync()?.get()?;
                let timeline = session.GetTimelineProperties()?;
                let playing = session.GetPlaybackInfo()?.PlaybackStatus()? == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing;
                let mut position = timeline.Position()?.Duration.max(0);
                if playing {
                    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
                    let now_ticks = now.as_secs() as i64 * 10_000_000 + now.subsec_nanos() as i64 / 100 + 116_444_736_000_000_000;
                    let elapsed = now_ticks.saturating_sub(timeline.LastUpdatedTime()?.UniversalTime).max(0);
                    position = position.saturating_add(elapsed);
                }
                let end = timeline.EndTime()?.Duration.max(0);
                if end > 0 { position = position.min(end); }
                Ok(Snapshot {
                    title: properties.Title()?.to_string(),
                    artist: properties.Artist()?.to_string(),
                    album: properties.AlbumTitle()?.to_string(),
                    playing,
                    position: (position / 10_000_000) as u32,
                    duration: (end / 10_000_000) as u32,
                    available: true,
                })
            })().unwrap_or_default();
            if output.send(snapshot).is_err() { return; }
        }
    });
    (commands, snapshots)
}
