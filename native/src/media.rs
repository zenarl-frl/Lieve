use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;
use windows::Media::Control::*;

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub artwork: Option<std::sync::Arc<crate::artwork::Image>>,
    pub artwork_pending: bool,
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
    #[test]
    #[ignore = "requires an active Windows media session with artwork"]
    fn current_session_artwork_decodes() {
        unsafe {crate::initialize_runtime();}
        let manager=super::GlobalSystemMediaTransportControlsSessionManager::RequestAsync().unwrap().get().unwrap();
        let properties=manager.GetCurrentSession().unwrap().TryGetMediaPropertiesAsync().unwrap().get().unwrap();
        let image=crate::artwork::load(&properties.Thumbnail().expect("Player must publish an album thumbnail")).unwrap();
        println!("Current session: {} / {}; artwork {}x{} ({} bytes)",properties.Title().unwrap(),properties.Artist().unwrap(),image.width,image.height,image.bgra.len());
        assert_eq!(image.bgra.len(),image.width as usize*image.height as usize*4);
    }
}

pub fn start() -> (Sender<Command>, Receiver<Snapshot>) {
    let (art_requests,art_updates)=crate::artwork::start();
    let (commands, input) = mpsc::channel();
    let (output, snapshots) = mpsc::channel();
    std::thread::spawn(move || {
        unsafe { super::initialize_runtime(); }
        let Ok(manager) = GlobalSystemMediaTransportControlsSessionManager::RequestAsync().and_then(|op| op.get()) else { return; };
        let mut artwork=crate::artwork::Cache::new();
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
                let title=properties.Title()?.to_string();let artist=properties.Artist()?.to_string();let album=properties.AlbumTitle()?.to_string();
                let key=format!("{:?}",(session.SourceAppUserModelId()?.to_string(),&title,&artist,&album,end));
                let now=std::time::Instant::now();artwork.track(&key,now);
                while let Ok(update)=art_updates.try_recv() {artwork.apply(update,now);}
                if artwork.due(now) {
                    if let Ok(thumbnail)=properties.Thumbnail().and_then(|t|windows::core::AgileReference::new(&t)) {
                        if art_requests.send(crate::artwork::Request{key,thumbnail}).is_ok() {artwork.requested();}
                        else {artwork.missing(now);}
                    } else {artwork.missing(now);}
                }
                Ok(Snapshot {
                    artwork:artwork.image.clone(),artwork_pending:artwork.pending(),title,artist,album,
                    playing,
                    position: (position / 10_000_000) as u32,
                    duration: (end / 10_000_000) as u32,
                    available: true,
                })
            })().unwrap_or_else(|_| {artwork.track("",std::time::Instant::now());Snapshot::default()});
            if output.send(snapshot).is_err() { return; }
        }
    });
    (commands, snapshots)
}
