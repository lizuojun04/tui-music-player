use crate::{
    app::{app::App}, 
    ui::components::{
        file_browser_drawer::FileBrowserDrawer, 
        playlist_drawer::PlaylistDrawer,
        music_info_drawer::MusicInfoDrawer
    }
}; 
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

pub trait Drawable {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: Rect);
}

pub struct UIDrawer {}

impl UIDrawer {
    pub fn drawn_ui(frame: &mut Frame, app: &mut App) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(40),
                Constraint::Percentage(40)
            ])
            .split(frame.area());
        FileBrowserDrawer::drawn_ui(frame, app, chunks[0]);
        PlaylistDrawer::drawn_ui(frame, app, chunks[1]);
        MusicInfoDrawer::drawn_ui(frame, app, chunks[2]);
    }
}
