use crate::{
    app::components::file_browser::FileBrowser,
    app::components::playlist::Playlist,
    app::event::{MainEvent, PlayerEvent},
    audio::client::PlayerClient,
    protocol::{PlayerRequest, PlayerStatus},
    ui::{theme, ui},
    utils::file_manager::FileManager,
};
use crossbeam_channel::{RecvTimeoutError, unbounded};
use ratatui::{
    Terminal,
    backend::Backend,
    widgets::{ListState, ScrollbarState, TableState},
};
use std::io;
use std::{path::PathBuf, time::Duration};

pub enum AppExit {
    Quit,
    Suspend,
}

pub struct CurrentSongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: f64,
}

impl CurrentSongInfo {
    pub fn change_info(&mut self, info_tuple: (String, String, String, f64)) {
        self.title = info_tuple.0;
        self.artist = info_tuple.1;
        self.album = info_tuple.2;
        self.duration = info_tuple.3;
    }
}

impl Default for CurrentSongInfo {
    fn default() -> Self {
        Self {
            title: "Unknown".to_string(),
            artist: "Unknown".to_string(),
            album: "Unknown".to_string(),
            duration: 0.0,
        }
    }
}

pub enum PlayOrder {
    Sequential,
    Shuffle,
}

/* enum CurrentScreen {
    MainScreen,
    FileBrowser,
    MusciPlayer,
} */

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ActiveBlock {
    FileBrowserBlock,
    PlaylistBlock,
    FilterNameBlock,
    FilterArtistBlock,
    FilterWorkBlock,
}

/// use to manage all states of the app,
/// leave the rendering logic to ui module,
/// all ui module are no-state
pub struct App {
    player: PlayerClient,
    player_status: PlayerStatus,

    pub activate_block: ActiveBlock,

    pub event_sender: crossbeam_channel::Sender<MainEvent>,
    event_receiver: crossbeam_channel::Receiver<MainEvent>,

    pub current_path: PathBuf,
    pub current_playing_song_path: Option<PathBuf>,
    pub current_playing_song_index: Option<usize>,

    pub file_browser: FileBrowser,
    file_browser_parent_index: usize,
    pub file_browser_list_state: ListState,

    pub playlist: Playlist,
    pub playlist_scroll_state: ScrollbarState,
    pub playlist_table_state: TableState,

    pub current_song_info: CurrentSongInfo,

    pub theme: theme::Theme,

    pub play_order: PlayOrder,

    pub filter_name_string: String,
    pub filter_artist_string: String,
    pub filter_work_string: String,

    pub filtered_playlist_indices: Vec<usize>,

    need_redraw: bool,
}

impl App {
    pub fn new(root_dir: PathBuf) -> Self {
        let (event_sender, event_receiver) = unbounded();
        let theme = theme::Theme::default();

        let file_browser =
            FileBrowser::from_paths(FileManager::get_entry_list_static(root_dir.clone()));
        let file_browser_list_state = ListState::default().with_selected(Some(0));

        let playlist =
            Playlist::from_paths(FileManager::get_file_path_list_static(root_dir.clone()));
        let playlist_scroll_state =
            ScrollbarState::new(playlist.items.len() * theme.playlist_theme.item_height);
        let playlist_table_state = TableState::new().with_selected(Some(0));
        let filtered_playlist_indices = (0..playlist.items.len()).collect();

        let mut app = Self {
            player: PlayerClient,
            player_status: PlayerStatus::default(),
            activate_block: ActiveBlock::PlaylistBlock,
            event_sender,
            event_receiver,
            current_path: root_dir,
            current_playing_song_path: None,
            current_playing_song_index: None,
            file_browser,
            file_browser_parent_index: 0,
            file_browser_list_state,
            playlist,
            playlist_scroll_state,
            playlist_table_state,
            current_song_info: CurrentSongInfo::default(),
            theme,
            play_order: PlayOrder::Sequential,
            filter_name_string: String::new(),
            filter_artist_string: String::new(),
            filter_work_string: String::new(),
            filtered_playlist_indices,
            need_redraw: true,
        };
        app.sync_playlist();
        app.refresh_player_status();
        app
    }

    pub fn run<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<AppExit>
    where
        B: Backend<Error = io::Error>,
    {
        loop {
            if self.need_redraw {
                terminal.draw(|frame| {
                    ui::UIDrawer::drawn_ui(frame, self);
                })?;
                self.need_redraw = false;
            }
            match self
                .event_receiver
                .recv_timeout(Duration::from_millis(1000))
            {
                Ok(MainEvent::Key(key)) => {
                    if key.code == crossterm::event::KeyCode::Char('z')
                        && key
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::CONTROL)
                    {
                        return Ok(AppExit::Suspend);
                    }
                    match self.activate_block {
                        ActiveBlock::PlaylistBlock => match key.code {
                            crossterm::event::KeyCode::Char('q') => {
                                return Ok(self.shutdown());
                            }
                            crossterm::event::KeyCode::Char('j') => self.next_playlist_item(),
                            crossterm::event::KeyCode::Char('k') => self.previous_playlist_item(),
                            crossterm::event::KeyCode::Char(';') => self.load_playlist_item(),
                            crossterm::event::KeyCode::Char(' ') => {
                                self.toggle_play_pause_playlist_item()
                            }
                            crossterm::event::KeyCode::Char('l') => self.play_next_song(),
                            crossterm::event::KeyCode::Char('h') => self.play_previous_song(),
                            crossterm::event::KeyCode::Char('f') => {
                                self.switch_to(ActiveBlock::FileBrowserBlock)
                            }
                            crossterm::event::KeyCode::Char('/') => {
                                self.switch_to(ActiveBlock::FilterNameBlock)
                            }
                            crossterm::event::KeyCode::Tab => self.toggle_play_order(),
                            crossterm::event::KeyCode::Char('i') => self.increase_volume(),
                            crossterm::event::KeyCode::Char('u') => self.decrease_volume(),
                            _ => {}
                        },
                        ActiveBlock::FileBrowserBlock => match key.code {
                            crossterm::event::KeyCode::Char('q') => {
                                return Ok(self.shutdown());
                            }
                            crossterm::event::KeyCode::Char('j') => self.next_file_browser_item(),
                            crossterm::event::KeyCode::Char('k') => {
                                self.previous_file_browser_item()
                            }
                            crossterm::event::KeyCode::Char('h') => self.parent_directory(),
                            crossterm::event::KeyCode::Char('l') => self.enter_directory(),
                            crossterm::event::KeyCode::Char('p') => {
                                self.switch_to(ActiveBlock::PlaylistBlock)
                            }
                            crossterm::event::KeyCode::Char('/') => {
                                self.switch_to(ActiveBlock::FilterNameBlock)
                            }
                            crossterm::event::KeyCode::Char('s') => self.set_pwd_as_playlist(),
                            _ => {}
                        },
                        ActiveBlock::FilterNameBlock
                        | ActiveBlock::FilterArtistBlock
                        | ActiveBlock::FilterWorkBlock => match key.code {
                            crossterm::event::KeyCode::Enter => {
                                self.switch_to(ActiveBlock::PlaylistBlock)
                            }
                            crossterm::event::KeyCode::Tab => self.toggle_filter_block(),
                            crossterm::event::KeyCode::Char(value) => self.push_string_input(value),
                            crossterm::event::KeyCode::Backspace => self.pop_string_input(),
                            _ => {}
                        },
                    }
                }
                Ok(MainEvent::Player(PlayerEvent::SongFinished)) => {
                    self.play_next_song();
                    self.need_redraw = true;
                }
                Ok(MainEvent::Player(PlayerEvent::SongInfo(info_tuple))) => {
                    self.current_song_info.change_info(info_tuple);
                    self.need_redraw = true;
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.refresh_player_status();
                    self.need_redraw = true;
                }
                Err(_) => return Ok(AppExit::Quit),
            }
        }
    }

    fn switch_to(&mut self, activate_block: ActiveBlock) {
        self.activate_block = activate_block;
        self.need_redraw = true;
    }

    fn toggle_filter_block(&mut self) {
        self.activate_block = match &self.activate_block {
            ActiveBlock::FilterNameBlock => ActiveBlock::FilterArtistBlock,
            ActiveBlock::FilterArtistBlock => ActiveBlock::FilterWorkBlock,
            ActiveBlock::FilterWorkBlock => ActiveBlock::FilterNameBlock,
            other => *other,
        };
        self.need_redraw = true;
    }

    fn push_string_input(&mut self, value: char) {
        match self.activate_block {
            ActiveBlock::FilterNameBlock => self.filter_name_string.push(value),
            ActiveBlock::FilterArtistBlock => self.filter_artist_string.push(value),
            ActiveBlock::FilterWorkBlock => self.filter_work_string.push(value),
            _ => {}
        }
        self.apply_filter();
        self.sync_playlist();
        self.playlist_scroll_state = self.playlist_scroll_state.content_length(
            self.filtered_playlist_indices.len() * self.theme.playlist_theme.item_height,
        );
        self.current_playing_song_index = None;
        self.playlist_table_state.select(Some(0));
        self.need_redraw = true;
    }

    fn pop_string_input(&mut self) {
        match self.activate_block {
            ActiveBlock::FilterNameBlock => {
                self.filter_name_string.pop();
            }
            ActiveBlock::FilterArtistBlock => {
                self.filter_artist_string.pop();
            }
            ActiveBlock::FilterWorkBlock => {
                self.filter_work_string.pop();
            }
            _ => {}
        }
        self.apply_filter();
        self.sync_playlist();
        self.playlist_scroll_state = self.playlist_scroll_state.content_length(
            self.filtered_playlist_indices.len() * self.theme.playlist_theme.item_height,
        );
        self.current_playing_song_index = None;
        self.playlist_table_state.select(Some(0));
        self.need_redraw = true;
    }

    fn toggle_play_order(&mut self) {
        self.play_order = match self.play_order {
            PlayOrder::Sequential => PlayOrder::Shuffle,
            PlayOrder::Shuffle => PlayOrder::Sequential,
        };
        let enabled = matches!(self.play_order, PlayOrder::Shuffle);
        let _ = self.player.request(&PlayerRequest::SetShuffle { enabled });
        self.need_redraw = true;
    }

    fn play_next_song(&mut self) {
        self.sync_playlist();
        let _ = self.player.request(&PlayerRequest::Next);
        self.refresh_player_status();
        self.need_redraw = true;
    }

    fn play_previous_song(&mut self) {
        self.sync_playlist();
        let _ = self.player.request(&PlayerRequest::Prev);
        self.refresh_player_status();
        self.need_redraw = true;
    }

    fn previous_playlist_item(&mut self) {
        if self.filtered_playlist_indices.is_empty() {
            return;
        }
        let selected = match self.playlist_table_state.selected() {
            Some(selected) => {
                if selected == 0 {
                    self.filtered_playlist_indices.len() - 1
                } else {
                    selected - 1
                }
            }
            None => 0,
        };
        self.playlist_table_state.select(Some(selected));
        self.playlist_scroll_state = self
            .playlist_scroll_state
            .position(selected * self.theme.playlist_theme.item_height);
        self.need_redraw = true;
    }

    fn next_playlist_item(&mut self) {
        if self.filtered_playlist_indices.is_empty() {
            return;
        }
        let selected = match self.playlist_table_state.selected() {
            Some(selected) => {
                if selected >= self.filtered_playlist_indices.len() - 1 {
                    0
                } else {
                    selected + 1
                }
            }
            None => 0,
        };
        self.playlist_table_state.select(Some(selected));
        self.playlist_scroll_state = self
            .playlist_scroll_state
            .position(selected * self.theme.playlist_theme.item_height);
        self.need_redraw = true;
    }

    fn load_playlist_item(&mut self) {
        if let Some(selected) = self.playlist_table_state.selected() {
            if selected >= self.filtered_playlist_indices.len() {
                return;
            }
            self.sync_playlist();
            self.current_playing_song_index = Some(selected);
            let song_path = self.playlist.items[self.filtered_playlist_indices[selected]]
                .get_file_path()
                .clone();
            self.current_playing_song_path = Some(song_path.clone());
            let _ = self
                .player
                .request(&PlayerRequest::Load { path: song_path });
            self.refresh_player_status();
        }
        self.need_redraw = true;
    }

    fn toggle_play_pause_playlist_item(&mut self) {
        let request = if self.player_status.is_playing {
            PlayerRequest::Pause
        } else {
            PlayerRequest::Continue
        };
        let _ = self.player.request(&request);
        self.refresh_player_status();
        self.need_redraw = true;
    }

    fn previous_file_browser_item(&mut self) {
        let selected = match self.file_browser_list_state.selected() {
            Some(selected) => {
                if selected == 0 {
                    self.file_browser.items.len() - 1
                } else {
                    selected - 1
                }
            }
            None => 0,
        };
        self.file_browser_list_state.select(Some(selected));
        self.need_redraw = true;
    }

    fn next_file_browser_item(&mut self) {
        let selected = match self.file_browser_list_state.selected() {
            Some(selected) => {
                if selected == self.file_browser.items.len() - 1 {
                    0
                } else {
                    selected + 1
                }
            }
            None => 0,
        };
        self.file_browser_list_state.select(Some(selected));
        self.need_redraw = true;
    }

    fn enter_directory(&mut self) {
        if let Some(selected) = self.file_browser_list_state.selected() {
            self.file_browser_parent_index = selected;
            let selected_path = self.file_browser.items[selected].get_file_path();
            if selected_path.is_dir() {
                self.current_path = selected_path.clone();
                self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(
                    self.current_path.clone(),
                ));
                self.file_browser_list_state.select(Some(0));
            }
        }
        self.need_redraw = true;
    }

    fn parent_directory(&mut self) {
        if let Some(parent_path) = self.current_path.parent() {
            self.current_path = parent_path.to_path_buf();
            self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(
                self.current_path.clone(),
            ));
            self.file_browser_list_state
                .select(Some(self.file_browser_parent_index));
        }
        self.need_redraw = true;
    }

    fn set_pwd_as_playlist(&mut self) {
        self.playlist = Playlist::from_paths(FileManager::get_file_path_list_static(
            self.current_path.clone(),
        ));
        self.filtered_playlist_indices = (0..self.playlist.items.len()).collect();
        self.playlist_table_state.select(Some(0));
        self.playlist_scroll_state = self.playlist_scroll_state.content_length(
            self.filtered_playlist_indices.len() * self.theme.playlist_theme.item_height,
        );
        self.current_playing_song_index = None;
        self.sync_playlist();
        self.need_redraw = true;
    }

    pub fn get_current_position(&self) -> f64 {
        self.player_status.position
    }

    pub fn is_playing(&self) -> bool {
        self.player_status.is_playing
    }

    pub fn increase_volume(&mut self) {
        let volume = self.player_status.volume;
        if volume < 100 {
            let _ = self
                .player
                .request(&PlayerRequest::SetVolume { volume: volume + 1 });
            self.refresh_player_status();
            self.need_redraw = true;
        }
    }

    pub fn decrease_volume(&mut self) {
        let volume = self.player_status.volume;
        if volume > 0 {
            let _ = self
                .player
                .request(&PlayerRequest::SetVolume { volume: volume - 1 });
            self.refresh_player_status();
            self.need_redraw = true;
        }
    }

    pub fn get_volume(&self) -> u32 {
        self.player_status.volume
    }

    fn apply_filter(&mut self) {
        let name_filter = self.filter_name_string.to_lowercase();
        let artist_filter = self.filter_artist_string.to_lowercase();
        let work_filter = self.filter_work_string.to_lowercase();

        let all_filters_empty =
            name_filter.is_empty() && artist_filter.is_empty() && work_filter.is_empty();

        if all_filters_empty {
            self.filtered_playlist_indices = (0..self.playlist.items.len()).collect();
        } else {
            self.filtered_playlist_indices = self
                .playlist
                .items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    let name_match = name_filter.is_empty()
                        || item.get_name().to_lowercase().contains(&name_filter);
                    let artist_match = artist_filter.is_empty()
                        || item.get_artist().to_lowercase().contains(&artist_filter);
                    let work_match = work_filter.is_empty()
                        || item.get_work().to_lowercase().contains(&work_filter);
                    name_match && artist_match && work_match
                })
                .map(|(index, _)| index)
                .collect();
        }
    }

    fn sync_playlist(&self) {
        let paths = self
            .filtered_playlist_indices
            .iter()
            .map(|&index| self.playlist.items[index].get_file_path().clone())
            .collect();
        let _ = self.player.request(&PlayerRequest::SetPlaylist { paths });
    }

    fn refresh_player_status(&mut self) {
        let Ok(status) = self.player.state() else {
            return;
        };
        self.current_playing_song_path = status.current_path.clone();
        self.current_playing_song_index = status.current_path.as_ref().and_then(|current| {
            self.filtered_playlist_indices
                .iter()
                .position(|&playlist_index| {
                    self.playlist.items[playlist_index].get_file_path() == current
                })
        });
        self.current_song_info.change_info((
            status.title.clone(),
            status.artist.clone(),
            status.album.clone(),
            status.duration,
        ));
        self.play_order = if status.shuffle {
            PlayOrder::Shuffle
        } else {
            PlayOrder::Sequential
        };
        self.player_status = status;
    }

    fn shutdown(&self) -> AppExit {
        let _ = self.player.request(&PlayerRequest::Shutdown);
        AppExit::Quit
    }
}
