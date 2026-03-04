use crate::{
    ui::ui::Drawable,
    app::app::App,
    utils::utils
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, palette::tailwind, Style},
    text::{Line, Span, Text},
    widgets::{Cell, Block, Borders, BorderType, Clear, List, LineGauge, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, Wrap}, 
    Frame
};

pub struct MusicInfoDrawer;

impl Drawable for MusicInfoDrawer {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: Rect) {
        // LOGO
        // picture
        // name
        // time
        // volume
        // 进度条
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(9),
                Constraint::Percentage(50),
                Constraint::Percentage(30),
                Constraint::Min(1)
            ])
            .split(Self::render_block_with_border(frame, app, area));
        Self::render_logo(frame, app, chunks[0]);
        Self::render_text_info(frame, app, chunks[2]);
        Self::render_progress_bar(frame, app, chunks[3]);
    }
}

impl MusicInfoDrawer {
    fn render_block_with_border(frame: &mut Frame, app: &mut App, area: Rect) -> Rect {
        let block = Block::default()
            .borders(app.theme.music_info_theme.music_info_borders)
            .title(Line::from("Music Playing").left_aligned())
            .border_type(app.theme.music_info_theme.music_info_border_type)
            .border_style(app.theme.music_info_theme.music_info_border_style);
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        inner_area
    }

    fn render_logo(frame: &mut Frame, app: &mut App, area: Rect) {
        let paragraph = Paragraph::new(app.theme.music_info_theme.logo)
            .style(app.theme.music_info_theme.logo_style)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_text_info(frame: &mut Frame, app: &mut App, area: Rect) {
        let rows = vec![
            Row::new(vec![Cell::from("  Name:"),     Cell::from(app.current_song_info.title.clone())]),
            Row::new(vec![Cell::from("  Artist:"),   Cell::from(app.current_song_info.artist.clone())]),
            Row::new(vec![Cell::from("  Album:"),    Cell::from(app.current_song_info.album.clone())]),
            Row::new(vec![Cell::from("  Duration:"), Cell::from(utils::format_duration(app.current_song_info.duration))]),
            Row::new(vec![Cell::from("  Time:"), Cell::from(utils::format_duration(app.get_current_position()))]),
        ];
        let widths = [Constraint::Length(11), Constraint::Min(10)];
        let table = Table::new(rows, widths)
            .column_spacing(app.theme.music_info_theme.table_column_spacing)
            .style(app.theme.music_info_theme.table_cell_style);
        frame.render_widget(table, area);
    }

    fn render_progress_bar(frame: &mut Frame, app: &mut App, area: Rect) {
        frame.render_widget(
            LineGauge::default()
                .filled_style(Style::new().white().on_black().bold())
                .unfilled_symbol("·")
                .filled_symbol("·")
                .filled_style(Style::new().blue().on_black().bold())
                .label(Line::raw(""))
                .ratio(0.4), 
            area
        );
    }

}
