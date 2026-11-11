use super::pipewire::{self, Offer, PwSource, Remote, Target};
use crate::source::FrameSource;
use ashpd::desktop::screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType};
use ashpd::desktop::PersistMode;
use std::fs;
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct PortalSource {
    inner: Option<PwSource>,
    keep: Option<thread::JoinHandle<()>>,
    stop: mpsc::Sender<()>,
}

pub fn open(token_path: &Path, offer: &Offer) -> Result<PortalSource, String> {
    let token = fs::read_to_string(token_path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty());
    let (ready_tx, ready_rx) = mpsc::sync_channel(1);
    let (stop_tx, stop_rx) = mpsc::channel();
    let token_path = token_path.to_owned();
    let keep = thread::Builder::new()
        .name("flat3d-portal".into())
        .spawn(move || {
            let error_tx = ready_tx.clone();
            let result = (|| -> Result<(), String> {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| e.to_string())?;
                runtime
                    .block_on(async move {
                        let proxy = Screencast::new().await.map_err(|e| e.to_string())?;
                        let session = proxy
                            .create_session(Default::default())
                            .await
                            .map_err(|e| e.to_string())?;
                        proxy
                            .select_sources(
                                &session,
                                SelectSourcesOptions::default()
                                    .set_cursor_mode(CursorMode::Hidden)
                                    .set_sources(SourceType::Monitor | SourceType::Window)
                                    .set_multiple(false)
                                    .set_restore_token(token.as_deref())
                                    .set_persist_mode(PersistMode::ExplicitlyRevoked),
                            )
                            .await
                            .map_err(|e| e.to_string())?
                            .response()
                            .map_err(|e| e.to_string())?;
                        let streams = proxy
                            .start(&session, None, Default::default())
                            .await
                            .map_err(|e| e.to_string())?
                            .response()
                            .map_err(|e| e.to_string())?;
                        let node = streams
                            .streams()
                            .first()
                            .ok_or_else(|| "portal returned no streams".to_owned())?
                            .pipe_wire_node_id();
                        if let Some(value) = streams.restore_token() {
                            write_token(&token_path, value).map_err(|e| e.to_string())?;
                        }
                        let fd = proxy
                            .open_pipe_wire_remote(&session, Default::default())
                            .await
                            .map_err(|e| e.to_string())?;
                        ready_tx.send(Ok((fd, node))).map_err(|e| e.to_string())?;
                        let _ = tokio::task::spawn_blocking(move || stop_rx.recv()).await;
                        Ok::<(), String>(())
                    })
                    .map_err(|e| e.to_string())
            })();
            // The successful path sends the remote before waiting; only errors remain to report here.
            if let Err(error) = result {
                let _ = error_tx.send(Err(error));
            }
        })
        .map_err(|e| e.to_string())?;
    let (fd, node) = ready_rx.recv().map_err(|e| e.to_string())??;
    let inner = match pipewire::open(Remote::Fd(fd), Target::NodeId(node), offer) {
        Ok(inner) => inner,
        Err(error) => {
            let _ = stop_tx.send(());
            let _ = keep.join();
            return Err(error);
        }
    };
    Ok(PortalSource {
        inner: Some(inner),
        keep: Some(keep),
        stop: stop_tx,
    })
}

fn write_token(path: &Path, token: &str) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.{}.tmp",
        path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("portal-token"),
        std::process::id()
    ));
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)?;
    use std::io::Write;
    file.write_all(token.as_bytes())?;
    file.sync_all()?;
    fs::rename(&tmp, path)?;
    Ok(())
}

impl FrameSource for PortalSource {
    fn next(&mut self, timeout: Duration) -> Result<Option<crate::frame::Frame>, String> {
        self.inner
            .as_mut()
            .ok_or_else(|| "portal stream is stopped".to_owned())?
            .next(timeout)
    }
}

impl Drop for PortalSource {
    fn drop(&mut self) {
        drop(self.inner.take());
        let _ = self.stop.send(());
        if let Some(keep) = self.keep.take() {
            let _ = keep.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn token_is_replaced_atomically_with_private_mode() {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("flat3d-portal-test-{id}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("restore-token");
        write_token(&path, "first").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "first");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        write_token(&path, "second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1);
        fs::remove_dir_all(dir).unwrap();
    }
}
