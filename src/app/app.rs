use crate::{
    audio::player::Player, 
    ui::{ui, theme}, 
    utils::file_manager::FileManager, 
    app::components::playlist::{Playlist, PlaylistItem}
};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering}
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, TableState}, 
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
    file_manager: FileManager,
    activate_block: ActiveBlock,

    pub playlist: Playlist,
    pub playlist_scroll_state: ScrollbarState,
    pub playlist_table_state: TableState,

    pub theme: theme::Theme
}

impl App {
    pub fn new(root_dir: PathBuf) -> Self {
        let file_manager = FileManager::new(root_dir);
        let theme = theme::Theme::default();
        let playlist = Playlist::from_paths(file_manager.get_file_path_list());
        let playlist_scroll_state = ScrollbarState::new(playlist.items.len() * theme.playlist_theme.item_height);
        let playlist_table_state = TableState::new().with_selected(Some(0));
        Self {
            player: Player::new(),
            file_manager,
            activate_block: ActiveBlock::PlaylistBlock,
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
            terminal.draw(|frame| {
                ui::UIDrawer::drawn_ui(frame, self);
                })?;

            // TODO 键位绑定不应该硬编码
            if crossterm::event::poll(std::time::Duration::from_millis(16))? {
                if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                    match key.code {
                        crossterm::event::KeyCode::Char('q') => return Ok(true),
                        crossterm::event::KeyCode::Char('j') => self.next_playlist_item(),
                        crossterm::event::KeyCode::Char('k') => self.previous_playlist_item(),
                        crossterm::event::KeyCode::Enter => self.load_playlist_item(),
                        crossterm::event::KeyCode::Char(' ') => self.toggle_play_pause_playlist_item(),
                        crossterm::event::KeyCode::Char('h') => {},
                        crossterm::event::KeyCode::Char('l') => {}
                        _ => {}
                    }
                }
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
    }

    fn next_playlist_item(&mut self) {
        let selected = match self.playlist_table_state.selected() {
            Some(selected) => {
                if selected == self.playlist.items.len() - 1 {
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

}
