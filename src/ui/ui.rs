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
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
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
        // let chunks = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([
        //         Constraint::Percentage(60),
        //         Constraint::Percentage(40)
        //     ])
        //     .split(frame.area());

        // let left_chunks = Layout::default()
        //     .direction(Direction::Vertical)
        //     .constraints([
        //         Constraint::Percentage(10),
        //         Constraint::Percentage(90)
        //     ])
        //     .split(chunks[0]);

        // let search_chunks = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([
        //         Constraint::Percentage(50),
        //         Constraint::Percentage(50)
        //     ])
        //     .split(left_chunks[0]);

        // let file_play_chunks = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([
        //         Constraint::Percentage(30),
        //         Constraint::Percentage(70)
        //     ])
        //     .split(left_chunks[1]);

        // PlaylistDrawer::drawn_ui(frame, app, file_play_chunks[1]);
    }
}

pub fn drawn_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(frame.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(90)
        ])
        .split(chunks[0]);

    let search_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(left_chunks[0]);

    let search_song_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Song")
        .style(Style::default());
    let search_song_paragraph = Paragraph::new(Text::styled("", Style::default()))
        .block(search_song_block);
    frame.render_widget(search_song_paragraph, search_chunks[0]);

    let search_artist_block = Block::default()
        .borders(Borders::ALL)
        .title("Search Artist")
        .style(Style::default());
    let search_artist_paragraph = Paragraph::new(Text::styled("", Style::default()))
        .block(search_artist_block);
    frame.render_widget(search_artist_paragraph, search_chunks[1]);

    let file_playlist_block = Block::default()
        .borders(Borders::ALL)
        .title("File Playlist")
        .style(Style::default());
    let file_playlist_paragraph = Paragraph::new(Text::styled("", Style::default()))
        .block(file_playlist_block);
    frame.render_widget(file_playlist_paragraph, left_chunks[1]);

    let music_player_block = Block::default()
        .borders(Borders::ALL)
        .title("Music Player")
        .style(Style::default());
    let music_player_paragraph = Paragraph::new(Text::styled("", Style::default()))
        .block(music_player_block);
    frame.render_widget(music_player_paragraph, chunks[1]);

}
