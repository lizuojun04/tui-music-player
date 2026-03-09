use crate::{
    ui::ui::Drawable,
    app::app::{App, PlayOrder},
    utils::utils
};

use ratatui::{
    buffer::{Buffer},
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Style},
    text::{Line},
    widgets::{Block, Cell, Paragraph, Row, StatefulWidget, Table}, 
    Frame
};

pub struct MusicInfoDrawer;

impl Drawable for MusicInfoDrawer {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: Rect) {
        // picture
        // volume
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),
                Constraint::Min(12),
                Constraint::Min(14),
                Constraint::Min(2),
                Constraint::Min(7),
                Constraint::Min(1),
                Constraint::Min(3)
            ])
            .split(Self::render_block_with_border(frame, app, area));
        Self::render_logo(frame, app,         chunks[1]);
        Self::render_picture(frame, app,      chunks[2]);
        Self::render_text_info(frame, app,    chunks[4]);
        Self::render_status(frame, app,       chunks[5]);
        Self::render_progress_bar(frame, app, chunks[6]);
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

    fn render_picture(frame: &mut Frame, app: &mut App, area: Rect) {
        let paragraph = Paragraph::new(app.theme.music_info_theme.picture)
            .style(app.theme.music_info_theme.logo_style)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, area);
    }

    fn render_text_info(frame: &mut Frame, app: &mut App, area: Rect) {
        let rows = vec![
            Row::new(vec![Cell::from(Line::from("Name").right_aligned()),     Cell::from(":"), Cell::from(app.current_song_info.title.clone())]),
            Row::new(vec![Cell::from(Line::from("Artist").right_aligned()),   Cell::from(":"), Cell::from(app.current_song_info.artist.clone())]),
            Row::new(vec![Cell::from(Line::from("Album").right_aligned()),    Cell::from(":"), Cell::from(app.current_song_info.album.clone())]),
            Row::new(vec![Cell::from(Line::from("Duration").right_aligned()), Cell::from(":"), Cell::from(utils::format_duration(app.current_song_info.duration))]),
        ];
        let widths = [Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)];
        let table = Table::new(rows, widths)
            .column_spacing(app.theme.music_info_theme.table_column_spacing)
            .style(app.theme.music_info_theme.table_cell_style);
        frame.render_widget(table, area);
    }

    fn render_status(frame: &mut Frame, app: &mut App, area: Rect) {
        let play_order_symbol = match app.play_order {
            PlayOrder::Sequential => app.theme.music_info_theme.play_order_sequential_symbol,
            PlayOrder::Shuffle => app.theme.music_info_theme.play_order_shuffle_symbol
        };
        let play_status_symbol = if app.is_playing() {
            app.theme.music_info_theme.play_status_playing_symbol
        } else {
            app.theme.music_info_theme.play_status_pause_symbol
        };

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((0.05 * 100.0) as u16),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Percentage((0.05 * 100.0) as u16),
            ])
            .split(area);

        frame.render_widget(
            Line::from(utils::format_duration(app.get_current_position()))
                .left_aligned()
                .style(app.theme.music_info_theme.play_status_style), 
            chunks[1]
        );

        frame.render_widget(
            Line::from(format!("{} {}", play_order_symbol, play_status_symbol))
                .centered()
                .style(app.theme.music_info_theme.play_status_style), 
            chunks[2]
        );

        frame.render_widget(
            Line::from(format!("{} {}%", app.theme.music_info_theme.play_status_volume_symbol, app.get_volume()))
                .right_aligned()
                .style(app.theme.music_info_theme.play_status_style), 
            chunks[3]
        );

    }

    fn render_progress_bar(frame: &mut Frame, app: &mut App, area: Rect) {
        let mut state = if app.current_song_info.duration == 0.0 {
            0.0
        } else {
            app.get_current_position() / app.current_song_info.duration
        };

        frame.render_stateful_widget(
            ProgressBar::new(
                app.theme.music_info_theme.progress_bar_filled_symbol,
                app.theme.music_info_theme.progress_bar_unfilled_symbol,
                app.theme.music_info_theme.progress_bar_head_symbol,
                app.theme.music_info_theme.progress_bar_start_symbol,
                app.theme.music_info_theme.progress_bar_end_symbol,
                app.theme.music_info_theme.progress_bar_filled_style,
                app.theme.music_info_theme.progress_bar_unfilled_style,
                app.theme.music_info_theme.progress_bar_head_style,
                0.05
            ),
            area, 
            &mut state
        );
    }
}

struct ProgressBar {
    filled_symbol: &'static str,
    unfilled_symbol: &'static str,
    head_symbol: &'static str,
    start_symbol: &'static str,
    end_symbol: &'static str,
    filled_style: Style,
    unfilled_style: Style,
    head_style: Style,
    one_side_blank_ratio: f64
}

impl ProgressBar {
    pub fn new(
        filled_symbol: &'static str,
        unfilled_symbol: &'static str,
        head_symbol: &'static str,
        start_symbol: &'static str,
        end_symbol: &'static str,
        filled_style: Style,
        unfilled_style: Style,
        head_style: Style,
        one_side_blank_ratio: f64
    ) -> Self {
        Self {
            filled_symbol,
            unfilled_symbol,
            head_symbol,
            start_symbol,
            end_symbol,
            filled_style,
            unfilled_style,
            head_style,
            one_side_blank_ratio
        }
    }
}

impl StatefulWidget for ProgressBar {
    type State = f64;
    
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width <= 2 {
            return;
        }

        let one_side_blank_length = (area.width as f64 * self.one_side_blank_ratio).floor() as u16;
        let start_pos = area.left()  + one_side_blank_length;
        let end_pos   = area.right() - one_side_blank_length - 1;

        if end_pos < start_pos + 2 {
            return;
        }

        let width = end_pos - start_pos - 1;
        let ratio = state.clamp(0.0, 1.0);

        let filled_len = ((width as f64 * ratio).floor() as u16).min(width - 1);
        let head_pos = start_pos + filled_len + 1;

        buf.cell_mut(Position::new(start_pos, area.top()))
            .unwrap()
            .set_symbol(self.start_symbol)
            .set_style(self.unfilled_style);

        for i in start_pos + 1 .. head_pos {
            let cell = buf.cell_mut(Position::new(i, area.top())).unwrap();
            cell.set_symbol(self.filled_symbol);
            cell.set_style(self.filled_style);
        }

        buf.cell_mut(Position::new(head_pos, area.top()))
            .unwrap()
            .set_symbol(self.head_symbol)
            .set_style(self.head_style);

        for i in head_pos + 1 .. end_pos {
            let cell = buf.cell_mut(Position::new(i, area.top())).unwrap();
            cell.set_symbol(self.unfilled_symbol);
            cell.set_style(self.unfilled_style);
        }

        buf.cell_mut(Position::new(end_pos, area.top()))
            .unwrap()
            .set_symbol(self.end_symbol)
            .set_style(self.unfilled_style);

    }
}
