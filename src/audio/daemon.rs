use crate::{
    app::event::{MainEvent, PlayerEvent},
    audio::{client::PlayerClient, player::Player},
    protocol::{PlayerRequest, PlayerResponse, PlayerStatus},
};
use crossbeam_channel::Receiver;
use rand::RngExt;
use std::{
    fs,
    io::{self, BufRead, BufReader, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
    sync::atomic::Ordering,
    thread,
    time::Duration,
};

pub struct PlayerDaemon {
    player: Player,
    event_receiver: Receiver<MainEvent>,
    playlist: Vec<PathBuf>,
    current_index: Option<usize>,
    current_path: Option<PathBuf>,
    song_info: (String, String, String, f64),
    shuffle: bool,
    running: bool,
}

impl PlayerDaemon {
    pub fn run() -> io::Result<()> {
        let socket_path = PlayerClient::socket_path();
        if PlayerClient.is_running() {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                format!("daemon is already running at {}", socket_path.display()),
            ));
        }
        if socket_path.exists() {
            fs::remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;
        listener.set_nonblocking(true)?;
        let _socket_guard = SocketGuard(socket_path);

        let (event_sender, event_receiver) = crossbeam_channel::unbounded();
        let mut daemon = Self {
            player: Player::new(event_sender),
            event_receiver,
            playlist: Vec::new(),
            current_index: None,
            current_path: None,
            song_info: ("Unknown".into(), "Unknown".into(), "Unknown".into(), 0.0),
            shuffle: false,
            running: true,
        };

        while daemon.running {
            while let Ok(event) = daemon.event_receiver.try_recv() {
                daemon.handle_event(event);
            }

            match listener.accept() {
                Ok((stream, _)) => daemon.handle_connection(stream)?,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn handle_connection(&mut self, mut stream: std::os::unix::net::UnixStream) -> io::Result<()> {
        let mut request = String::new();
        BufReader::new(stream.try_clone()?).read_line(&mut request)?;
        let response = match serde_json::from_str::<PlayerRequest>(&request) {
            Ok(request) => self.handle_request(request),
            Err(error) => PlayerResponse::error(format!("invalid request: {error}")),
        };
        serde_json::to_writer(&mut stream, &response).map_err(io::Error::other)?;
        stream.write_all(b"\n")?;
        Ok(())
    }

    fn handle_request(&mut self, request: PlayerRequest) -> PlayerResponse {
        match request {
            PlayerRequest::Ping => PlayerResponse::ok(),
            PlayerRequest::GetState => PlayerResponse::state(self.status()),
            PlayerRequest::SetPlaylist { paths } => {
                self.playlist = paths;
                self.current_index = self
                    .current_path
                    .as_ref()
                    .and_then(|current| self.playlist.iter().position(|path| path == current));
                PlayerResponse::ok()
            }
            PlayerRequest::Load { path } => {
                self.current_index = self.playlist.iter().position(|item| item == &path);
                self.current_path = Some(path.clone());
                self.player.load(path);
                PlayerResponse::ok()
            }
            PlayerRequest::Continue => {
                self.player.play();
                PlayerResponse::ok()
            }
            PlayerRequest::Pause => {
                self.player.pause();
                PlayerResponse::ok()
            }
            PlayerRequest::Shutdown => {
                self.player.stop();
                self.running = false;
                PlayerResponse::ok()
            }
            PlayerRequest::Next => self.change_song(Direction::Next),
            PlayerRequest::Prev => self.change_song(Direction::Prev),
            PlayerRequest::SetVolume { volume } if volume <= 100 => {
                self.player.set_volume(volume);
                PlayerResponse::ok()
            }
            PlayerRequest::SetVolume { .. } => {
                PlayerResponse::error("volume must be between 0 and 100")
            }
            PlayerRequest::SetShuffle { enabled } => {
                self.shuffle = enabled;
                PlayerResponse::ok()
            }
        }
    }

    fn handle_event(&mut self, event: MainEvent) {
        match event {
            MainEvent::Player(PlayerEvent::SongFinished) => {
                let _ = self.change_song(Direction::Next);
            }
            MainEvent::Player(PlayerEvent::SongInfo(info)) => self.song_info = info,
            MainEvent::Key(_) => {}
        }
    }

    fn change_song(&mut self, direction: Direction) -> PlayerResponse {
        if self.playlist.is_empty() {
            return PlayerResponse::error("playlist is empty");
        }

        let index = if self.shuffle {
            rand::rng().random_range(0..self.playlist.len())
        } else {
            match (direction, self.current_index) {
                (Direction::Next, Some(index)) => (index + 1) % self.playlist.len(),
                (Direction::Prev, Some(0)) => self.playlist.len() - 1,
                (Direction::Prev, Some(index)) => index - 1,
                (_, None) => 0,
            }
        };
        let path = self.playlist[index].clone();
        self.current_index = Some(index);
        self.current_path = Some(path.clone());
        self.player.load(path);
        PlayerResponse::ok()
    }

    fn status(&self) -> PlayerStatus {
        PlayerStatus {
            is_playing: self.player.state.is_playing.load(Ordering::Relaxed),
            volume: self.player.state.volume.load(Ordering::Relaxed),
            position: self.player.get_current_position(),
            current_path: self.current_path.clone(),
            title: self.song_info.0.clone(),
            artist: self.song_info.1.clone(),
            album: self.song_info.2.clone(),
            duration: self.song_info.3,
            shuffle: self.shuffle,
        }
    }
}

enum Direction {
    Next,
    Prev,
}

struct SocketGuard(PathBuf);

impl Drop for SocketGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
