use crate::protocol::{PlayerRequest, PlayerResponse, PlayerStatus};
use std::{
    env,
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
};

#[derive(Clone, Copy, Default)]
pub struct PlayerClient;

impl PlayerClient {
    pub fn socket_path() -> PathBuf {
        if let Some(runtime_dir) = env::var_os("XDG_RUNTIME_DIR") {
            return PathBuf::from(runtime_dir).join("tui-music-player.sock");
        }

        let user = env::var("USER")
            .unwrap_or_else(|_| "unknown".to_string())
            .replace(|character: char| !character.is_ascii_alphanumeric(), "_");
        PathBuf::from(format!("/tmp/tui-music-player-{user}.sock"))
    }

    pub fn request(&self, request: &PlayerRequest) -> io::Result<PlayerResponse> {
        let mut stream = UnixStream::connect(Self::socket_path())?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        serde_json::to_writer(&mut stream, request).map_err(io::Error::other)?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut response = String::new();
        BufReader::new(stream).read_line(&mut response)?;
        let response: PlayerResponse = serde_json::from_str(&response).map_err(io::Error::other)?;
        if let Some(error) = response.error.as_deref() {
            return Err(io::Error::other(error.to_string()));
        }
        Ok(response)
    }

    pub fn is_running(&self) -> bool {
        self.request(&PlayerRequest::Ping).is_ok()
    }

    pub fn state(&self) -> io::Result<PlayerStatus> {
        self.request(&PlayerRequest::GetState)?
            .state
            .ok_or_else(|| io::Error::other("daemon returned no player state"))
    }
}
