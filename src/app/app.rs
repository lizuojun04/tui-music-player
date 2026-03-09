use crate::{
    audio::player::{Player},
    ui::{ui, theme}, 
    utils::{
        file_manager::FileManager, 
        key_input::KeyInput
    },
    app::components::playlist::{Playlist},
    app::components::file_browser::{FileBrowser},
    app::event::{MainEvent, PlayerEvent}
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}, time::Duration
};
use crossbeam_channel::{unbounded, RecvTimeoutError};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{ListState, ScrollbarState, TableState}, 
    Terminal,
};
use rand::RngExt;
use std::io;

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
    Shuffle
}

enum CurrentScreen {
    MainScreen,
    FileBrowser,
    MusciPlayer,
}

enum ActiveBlock {
    FileBrowserBlock,
    PlaylistBlock,
    SearchSongBlock,
    SearchArtistBlock
}

/// use to manage all states of the app, 
/// leave the rendering logic to ui module, 
/// all ui module are no-state
pub struct App {
    player: Player,
    activate_block: ActiveBlock,

    pub event_sender: crossbeam_channel::Sender<MainEvent>,
    event_receiver: crossbeam_channel::Receiver<MainEvent>,

    pub current_path: PathBuf,
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

    need_redraw: bool
}

impl App {
    pub fn new(root_dir: PathBuf) -> Self {
        let (event_sender, event_receiver) = unbounded();
        let theme = theme::Theme::default();

        let file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(root_dir.clone()));
        let file_browser_list_state = ListState::default().with_selected(Some(0));

        let playlist = Playlist::from_paths(FileManager::get_file_path_list_static(root_dir.clone()));
        let playlist_scroll_state = ScrollbarState::new(playlist.items.len() * theme.playlist_theme.item_height);
        let playlist_table_state = TableState::new().with_selected(Some(0));

        Self {
            player: Player::new(event_sender.clone()),
            activate_block: ActiveBlock::PlaylistBlock,
            event_sender,
            event_receiver,
            current_path: root_dir,
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
            need_redraw: true
        }
    }

    pub fn run<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<bool> 
        where 
            B: Backend<Error = io::Error>
    {
        KeyInput::listen_key_input(self.event_sender.clone());

        loop {
            if self.need_redraw {
                terminal.draw(|frame| {
                    ui::UIDrawer::drawn_ui(frame, self);
                })?;
                self.need_redraw = false;
            }
            match self.event_receiver.recv_timeout(Duration::from_millis(1000)) {
                Ok(MainEvent::Key(key)) => {
                    match self.activate_block {
                        ActiveBlock::PlaylistBlock => {
                            match key.code {
                                crossterm::event::KeyCode::Char('q') => return Ok(true),
                                crossterm::event::KeyCode::Char('j') => self.next_playlist_item(),
                                crossterm::event::KeyCode::Char('k') => self.previous_playlist_item(),
                                crossterm::event::KeyCode::Char(';') => self.load_playlist_item(),
                                crossterm::event::KeyCode::Char(' ') => self.toggle_play_pause_playlist_item(),
                                crossterm::event::KeyCode::Char('l') => self.play_next_song(),
                                crossterm::event::KeyCode::Char('h') => self.play_previous_song(),
                                crossterm::event::KeyCode::Char('f') => self.switch_to(ActiveBlock::FileBrowserBlock),
                                crossterm::event::KeyCode::Tab => self.toggle_play_order(),
                                _ => {}
                            }
                        },
                        ActiveBlock::FileBrowserBlock => {
                            match key.code {
                                crossterm::event::KeyCode::Char('q') => return Ok(true),
                                crossterm::event::KeyCode::Char('j') => self.next_file_browser_item(),
                                crossterm::event::KeyCode::Char('k') => self.previous_file_browser_item(),
                                crossterm::event::KeyCode::Char('h') => self.parent_directory(),
                                crossterm::event::KeyCode::Char('l') => self.enter_directory(),
                                crossterm::event::KeyCode::Char('p') => self.switch_to(ActiveBlock::PlaylistBlock),
                                crossterm::event::KeyCode::Char('s') => self.set_pwd_as_playlist(),
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
                Ok(MainEvent::Player(PlayerEvent::SongFinished)) => {
                    self.play_next_song();
                    self.need_redraw = true;
                },
                Ok(MainEvent::Player(PlayerEvent::SongInfo(info_tuple))) => {
                    self.current_song_info.change_info(info_tuple);
                    self.need_redraw = true;
                },
                Err(RecvTimeoutError::Timeout) => {
                    self.need_redraw = true;
                },
                Err(_) => return Ok(false)
            }
        }
    }

    fn switch_to(&mut self, activate_block: ActiveBlock) {
        self.activate_block = activate_block;
        self.need_redraw = true;
    }

    fn toggle_play_order(&mut self) {
        self.play_order = match self.play_order {
            PlayOrder::Sequential => PlayOrder::Shuffle,
            PlayOrder::Shuffle => PlayOrder::Sequential
        };
        self.need_redraw = true;
    }

    fn play_next_song(&mut self) {
        let index = self.get_next_index();
        self.player.load(self.playlist.items[index].get_file_path().clone());
        self.current_playing_song_index = Some(index);
        self.need_redraw = true;
    }

    fn play_previous_song(&mut self) {
        let index = self.current_playing_song_index.unwrap();
        self.player.load(self.playlist.items[index].get_file_path().clone());
        self.current_playing_song_index = Some(index);
        self.need_redraw = true;
    }

    fn get_next_index(&self) -> usize {
        match self.play_order {
            PlayOrder::Sequential => {
                match self.current_playing_song_index {
                    Some(index) => {
                        if index >= self.playlist.items.len() - 1 {
                            0
                        } else {
                            index + 1
                        }
                    }
                    None => {
                        0
                    }
                }
            },
            PlayOrder::Shuffle => {
                let mut rng = rand::rng();
                rng.random_range(0..self.playlist.items.len())
            }
        }
    }

    fn get_previous_index(&self) -> usize {
        match self.play_order {
            PlayOrder::Sequential => {
                match self.current_playing_song_index {
                    Some(index) => {
                        if index == 0 {
                            self.playlist.items.len() - 1
                        } else {
                            index - 1
                        }
                    }
                    None => {
                        0
                    }
                }
            },
            PlayOrder::Shuffle => {
                let mut rng = rand::rng();
                rng.random_range(0..self.playlist.items.len())
            }
        }
    }

    fn previous_playlist_item(&mut self) {
        let selected = match self.playlist_table_state.selected() {
            Some(selected) => {
                if selected == 0 {
                    self.playlist.items.len() - 1
                } else {
                    selected - 1
                }
            }
            None => 0,
        };
        self.playlist_table_state.select(Some(selected));
        self.playlist_scroll_state = self.playlist_scroll_state.position(selected * self.theme.playlist_theme.item_height);
        self.need_redraw = true;
    }

    fn next_playlist_item(&mut self) {
        let selected = match self.playlist_table_state.selected() {
            Some(selected) => {
                if selected >= self.playlist.items.len() - 1 {
                    0
                } else {
                    selected + 1
                }
            }
            None => 0,
        };
        self.playlist_table_state.select(Some(selected));
        self.playlist_scroll_state = self.playlist_scroll_state.position(selected * self.theme.playlist_theme.item_height);
        self.need_redraw = true;
    }

    fn load_playlist_item(&mut self) {
        if let Some(selected) = self.playlist_table_state.selected() {
            self.current_playing_song_index = Some(selected);
            self.player.load(self.playlist.items[selected].get_file_path().clone());
        }
        self.need_redraw = true;
    }

    fn toggle_play_pause_playlist_item(&mut self) {
        if self.player.state.is_playing.load(Ordering::Relaxed) {
            self.player.pause();
        } else {
            self.player.play();
        }
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
                self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(self.current_path.clone()));
                self.file_browser_list_state.select(Some(0));
            }
        }
        self.need_redraw = true;
    }

    fn parent_directory(&mut self) {
        if let Some(parent_path) = self.current_path.parent() {
            self.current_path = parent_path.to_path_buf();
            self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(self.current_path.clone()));
            self.file_browser_list_state.select(Some(self.file_browser_parent_index));
        }
        self.need_redraw = true;
    }

    /// WARN 如何处理 current_playing_song_index
    fn set_pwd_as_playlist(&mut self) {
        self.playlist = Playlist::from_paths(FileManager::get_file_path_list_static(self.current_path.clone()));
        self.playlist_table_state.select(Some(0));
        self.playlist_scroll_state = self.playlist_scroll_state.position(0);
        self.current_playing_song_index = None;
        self.need_redraw = true;
    }

    pub fn get_current_position(&self) -> f64 {
        self.player.get_current_position()
    }

    pub fn is_playing(&self) -> bool {
        self.player.state.is_playing.load(Ordering::Relaxed)
    }

}
