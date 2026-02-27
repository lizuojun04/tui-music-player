use crate::ui::ui::Drawable;
use crate::app::{
    components::playlist::{Playlist, PlaylistItem},
    app::App
};
use ratatui::style::Stylize;
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
        Self::render_table(frame, app, area);
        Self::render_scrollbar(frame, app, area);
    }
}

impl PlaylistDrawer {
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
        let block = Block::default()
            .borders(app.theme.playlist_theme.table_borders)
            .title("Playlist")
            .border_type(app.theme.playlist_theme.table_border_type)
            .border_style(app.theme.playlist_theme.table_border_style)
            .style(Style::default());
        let table = Table::new(rows, widths)
            .column_spacing(app.theme.playlist_theme.table_column_spacing)
            .header(header)
            .row_highlight_style(app.theme.playlist_theme.table_row_highlight_style)
            .highlight_symbol(app.theme.playlist_theme.table_highlight_symbol)
            .block(block)
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
            area.inner(Margin {
                vertical: 1,
                horizontal: 2
            }),
            &mut app.playlist_scroll_state
        )
    }
}

