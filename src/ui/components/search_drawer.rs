use crate::{
    ui::ui::Drawable,
    app::app::{App, ActiveBlock},
};

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span},
    widgets::{Block, Paragraph}, 
    Frame
};

pub struct SearchDrawer;

impl Drawable for SearchDrawer {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 3); 3])
            .split(area);
        Self::render_name_search(frame, app, chunks[0]);
        Self::render_artist_search(frame, app, chunks[1]);
        Self::render_work_search(frame, app, chunks[2]);
    }
}

impl SearchDrawer {
    fn render_name_search(frame: &mut Frame, app: &mut App, area: Rect) {
        let filter_string = app.filter_name_string.clone();
        let is_active = app.activate_block == ActiveBlock::FilterNameBlock;
        Self::render_search_block(frame, app, area, "Search by Name", &filter_string, is_active);
    }

    fn render_artist_search(frame: &mut Frame, app: &mut App, area: Rect) {
        let filter_string = app.filter_artist_string.clone();
        let is_active = app.activate_block == ActiveBlock::FilterArtistBlock;
        Self::render_search_block(frame, app, area, "Search by Artist", &filter_string, is_active);
    }

    fn render_work_search(frame: &mut Frame, app: &mut App, area: Rect) {
        let filter_string = app.filter_work_string.clone();
        let is_active = app.activate_block == ActiveBlock::FilterWorkBlock;
        Self::render_search_block(frame, app, area, "Search by Work", &filter_string, is_active);
    }

    fn render_search_block(frame: &mut Frame, app: &mut App, area: Rect, title: &str, filter_string: &str, is_active: bool) {
        let block = Block::default()
            .borders(app.theme.music_info_theme.music_info_borders)
            .title(Line::from(title).left_aligned())
            .border_type(app.theme.music_info_theme.music_info_border_type)
            .border_style(app.theme.music_info_theme.music_info_border_style);
        let filter_text = Span::raw(format!(" {}", filter_string));
        let filter_symbol = if is_active {
            Span::raw(app.theme.search_theme.search_block_selected_symbol)
                .style(app.theme.search_theme.search_block_selected_style)
        } else {
            Span::raw(" ")
        };
        let text = Paragraph::new(Line::from(vec![filter_symbol, filter_text]))
            .left_aligned()
            .block(block);
        frame.render_widget(text, area);
    }

}
