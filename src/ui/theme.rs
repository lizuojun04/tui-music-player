use ratatui::widgets::{Borders, BorderType};

use ratatui::{
    style::{self, Color, palette::tailwind}
};

pub struct Theme {
    pub playlist_theme: PlaylistTheme
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            playlist_theme: PlaylistTheme::default()
        }
    }
}

pub struct PlaylistTheme {
    pub table_cell_style: style::Style,
    pub table_header_style: style::Style,
    pub table_borders: Borders,
    pub table_border_type: BorderType,
    pub table_border_style: style::Style,
    pub table_column_spacing: u16,
    pub table_row_highlight_style: style::Style,
    pub table_highlight_symbol: &'static str,
    pub table_style: style::Style,
    pub scrollbar_begin_symbol: &'static str,
    pub scrollbar_track_symbol: &'static str,
    pub scrollbar_track_style: style::Style,
    pub scrollbar_thumb_symbol: &'static str,
    pub scrollbar_thumb_style: style::Style,
    pub scrollbar_end_symbol: &'static str,
    // pub scrollbar_style: style::Style,
    pub item_height: usize
}

impl Default for PlaylistTheme {
    fn default() -> Self {
        Self {
            table_cell_style: style::Style::default(),
            table_header_style: style::Style::default().fg(const_colors::GRAY_BLUE).bold(),
            table_borders: Borders::ALL,
            table_border_type: BorderType::Rounded,
            table_border_style: style::Style::default().fg(const_colors::LIGHT_BLUE),
            table_column_spacing: 2,
            table_row_highlight_style: style::Style::default().fg(const_colors::GREEN).bold(),
            table_highlight_symbol: " ",
            table_style: style::Style::default().fg(const_colors::DARK_GRAY_BLUE),
            scrollbar_begin_symbol: "",// "󰗴",
            scrollbar_track_symbol: "󱪼",
            scrollbar_track_style: style::Style::default().fg(const_colors::DARK_GRAY_BLUE),
            scrollbar_thumb_symbol: "󱪽",
            scrollbar_thumb_style: style::Style::default().fg(const_colors::BLUE),
            scrollbar_end_symbol: "",
            // scrollbar_style: style::Style::default().fg(const_colors::DARK_GREEN),
            item_height: 4
        }
    }
}

pub mod const_colors {
    use super::Color;
    
    pub const LIGHT_BLUE:  Color = Color::Rgb(58, 157, 214);
    pub const BLUE: Color = Color::Rgb(7, 129, 229);
    pub const GREEN: Color = Color::Rgb(21, 194, 146);
    pub const DARK_GREEN: Color = Color::Rgb(20, 150, 100);
    pub const GRAY_BLUE:  Color = Color::Rgb(150, 180, 250);
    pub const DARK_GRAY:  Color = Color::Rgb(120, 120, 120);
    pub const DARK_GRAY_BLUE:  Color = Color::Rgb(90, 110, 150);
}
