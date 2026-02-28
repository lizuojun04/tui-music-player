use crate::{
    audio::player::{Player, PlayerEvent},
    ui::{ui, theme}, 
    utils::file_manager::FileManager, 
    app::components::playlist::{Playlist},
    app::components::file_browser::{FileBrowser}
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}
};
use crossbeam_channel::unbounded;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState}, 
    Terminal,
};

use std::io;

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

    event_receiver: crossbeam_channel::Receiver<PlayerEvent>,

    pub current_path: PathBuf,
    pub current_playing_song_index: Option<usize>,

    pub file_browser: FileBrowser,
    pub file_browser_list_state: ListState,

    pub playlist: Playlist,
    pub playlist_scroll_state: ScrollbarState,
    pub playlist_table_state: TableState,

    pub theme: theme::Theme
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
            player: Player::new(event_sender),
            activate_block: ActiveBlock::PlaylistBlock,
            event_receiver,
            current_path: root_dir,
            current_playing_song_index: None,
            file_browser,
            file_browser_list_state,
            playlist,
            playlist_scroll_state,
            playlist_table_state,
            theme
        }
    }

    pub fn run<B>(&mut self, terminal: &mut Terminal<B>) -> io::Result<bool> 
        where 
            B: Backend<Error = io::Error>
    {
        loop {
            if let Ok(event) = self.event_receiver.try_recv() {
                match event {
                    PlayerEvent::SongFinished => {
                        self.play_next_song();
                    }
                }
            }

            terminal.draw(|frame| {
                ui::UIDrawer::drawn_ui(frame, self);
            })?;

            // TODO 键位绑定不应该硬编码
            if crossterm::event::poll(std::time::Duration::from_millis(16))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {

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
            }
        }
    }

    fn switch_to(&mut self, activate_block: ActiveBlock) {
        self.activate_block = activate_block;
    }

    fn play_next_song(&mut self) {
        match self.current_playing_song_index {
            Some(index) => {
                if index >= self.playlist.items.len() - 1 {
                    self.current_playing_song_index = Some(0);
                } else {
                    self.current_playing_song_index = Some(index + 1);
                }
            }
            None => {
                self.current_playing_song_index = Some(0);
            }
        }
        self.player.load(self.playlist.items[self.current_playing_song_index.unwrap()].get_file_path().clone());
    }

    fn play_previous_song(&mut self) {
        match self.current_playing_song_index {
            Some(index) => {
                if index == 0 {
                    self.current_playing_song_index = Some(self.playlist.items.len() - 1);
                } else {
                    self.current_playing_song_index = Some(index - 1);
                }
            }
            None => {
                self.current_playing_song_index = Some(0);
            }
        }
        self.player.load(self.playlist.items[self.current_playing_song_index.unwrap()].get_file_path().clone());
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
    }

    fn load_playlist_item(&mut self) {
        if let Some(selected) = self.playlist_table_state.selected() {
            self.current_playing_song_index = Some(selected);
            self.player.load(self.playlist.items[selected].get_file_path().clone());
        }
    }

    fn toggle_play_pause_playlist_item(&mut self) {
        if self.player.state.is_playing.load(Ordering::Relaxed) {
            self.player.pause();
        } else {
            self.player.play();
        }
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
    }

    fn enter_directory(&mut self) {
        if let Some(selected) = self.file_browser_list_state.selected() {
            let selected_path = self.file_browser.items[selected].get_file_path();
            if selected_path.is_dir() {
                self.current_path = selected_path.clone();
                self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(self.current_path.clone()));
                self.file_browser_list_state.select(Some(0));
            }
        }
    }

    fn parent_directory(&mut self) {
        if let Some(parent_path) = self.current_path.parent() {
            self.current_path = parent_path.to_path_buf();
            self.file_browser = FileBrowser::from_paths(FileManager::get_entry_list_static(self.current_path.clone()));
            self.file_browser_list_state.select(Some(0));
        }
    }

    /// WARN 如何处理 current_playing_song_index
    fn set_pwd_as_playlist(&mut self) {
        self.playlist = Playlist::from_paths(FileManager::get_file_path_list_static(self.current_path.clone()));
        self.playlist_table_state.select(Some(0));
        self.playlist_scroll_state = self.playlist_scroll_state.position(0);
        self.current_playing_song_index = None;
    }

}
