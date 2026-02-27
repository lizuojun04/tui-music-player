use crate::ui::ui::Drawable;
use crate::app::{
    components::playlist::{Playlist, PlaylistItem},
    app::App
};
use ratatui::style::Stylize;
use ratatui::widgets::TitlePosition;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, palette::tailwind, Style},
    text::{Line, Span, Text},
    widgets::{Cell, Block, Borders, BorderType, Clear, List, ListItem, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap}, 
    Frame
};

pub struct PlaylistDrawer {
}

// TODO 这里也硬编码了
// TODO 样式设计
impl Drawable for PlaylistDrawer {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(2)
            ])
            .split(Self::render_block_with_border(frame, app, area));
        Self::render_table(frame, app, chunks[0]);
        Self::render_scrollbar(frame, app, chunks[1]);
    }
}

impl PlaylistDrawer {
    fn render_block_with_border(frame: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) -> ratatui::prelude::Rect{
        let block = Block::default()
            .borders(app.theme.playlist_theme.table_borders)
            .title(Line::from("Playlist").left_aligned())
            .title(
                Line::from(
                    match app.current_playing_song_index {
                        Some(index) => {
                            let item = &app.playlist.items[index];
                            item.name.to_string()
                        },
                        None => "waiting for a song".to_string()
                    }
                ).right_aligned()
            )
            .border_type(app.theme.playlist_theme.table_border_type)
            .border_style(app.theme.playlist_theme.table_border_style);
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        inner_area
    }
    fn render_table(frame: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) {
        let rows = app.playlist.items.iter().map(|item| {
            let style = app.theme.playlist_theme.table_cell_style;
            Row::new(vec![
                Cell::from(item.name.clone()).style(style),
                Cell::from(item.artist.clone()).style(style),
                Cell::from(item.work.clone()).style(style)
            ])
        }).collect::<Vec<_>>();
        let widths = [Constraint::Percentage(60), Constraint::Percentage(20), Constraint::Percentage(20)];
        let header_style = app.theme.playlist_theme.table_header_style;
        let header = Row::new(vec![
            Cell::from("Name").style(header_style),
            Cell::from("Artist").style(header_style),
            Cell::from("Work").style(header_style)
        ]);
        let table = Table::new(rows, widths)
            .column_spacing(app.theme.playlist_theme.table_column_spacing)
            .header(header)
            .row_highlight_style(app.theme.playlist_theme.table_row_highlight_style)
            .highlight_symbol(app.theme.playlist_theme.table_highlight_symbol)
            .style(app.theme.playlist_theme.table_style);
        frame.render_stateful_widget(table, area, &mut app.playlist_table_state);
    }

    fn render_scrollbar(frame: &mut Frame, app: &mut App, area: ratatui::prelude::Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some(app.theme.playlist_theme.scrollbar_begin_symbol))
                .track_symbol(Some(app.theme.playlist_theme.scrollbar_track_symbol))
                .track_style(app.theme.playlist_theme.scrollbar_track_style)
                .thumb_symbol(app.theme.playlist_theme.scrollbar_thumb_symbol)
                .thumb_style(app.theme.playlist_theme.scrollbar_thumb_style)
                .end_symbol(Some(app.theme.playlist_theme.scrollbar_end_symbol)),
            area,
            &mut app.playlist_scroll_state
        )
    }
}

