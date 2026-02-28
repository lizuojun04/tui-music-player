use ratatui::widgets::{Borders, BorderType};

use ratatui::{
    style::{self, Color, palette::tailwind}
};

#[derive(Default)]
pub struct Theme {
    pub playlist_theme: PlaylistTheme,
    pub file_browser_theme: FileBrowserTheme
}

pub struct PlaylistTheme {
    pub table_cell_style: style::Style,
    pub table_header_style: style::Style,
    pub playlist_borders: Borders,
    pub playlist_border_type: BorderType,
    pub playlist_border_style: style::Style,
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
            playlist_borders: Borders::ALL,
            playlist_border_type: BorderType::Rounded,
            playlist_border_style: style::Style::default().fg(const_colors::LIGHT_BLUE),
            table_column_spacing: 2,
            table_row_highlight_style: style::Style::default()
                .bg(const_colors::LIGHT_GRAY)
                .fg(const_colors::GREEN)
                .bold(),
            table_highlight_symbol: " ",
            table_style: style::Style::default().fg(const_colors::DARK_GRAY_BLUE),
            scrollbar_begin_symbol: "",// "󰗴",
            scrollbar_track_symbol: "󱪼",
            scrollbar_track_style: style::Style::default().fg(const_colors::DARK_GRAY_BLUE),
            scrollbar_thumb_symbol: "󱪽",
            scrollbar_thumb_style: style::Style::default().fg(const_colors::BLUE),
            scrollbar_end_symbol: "",
            // scrollbar_style: style::Style::default(),
            item_height: 1
        }
    }
}

pub struct FileBrowserTheme {
    pub file_browser_borders: Borders,
    pub file_browser_border_type: BorderType,
    pub file_browser_border_style: style::Style,
    pub list_file_style: style::Style,
    pub list_directory_style: style::Style,
    pub list_highlight_style: style::Style,
    pub list_highlight_symbol: &'static str
}

impl Default for FileBrowserTheme {
    fn default() -> Self {
        Self {
            file_browser_borders: Borders::ALL,
            file_browser_border_type: BorderType::Rounded,
            file_browser_border_style: style::Style::default().fg(const_colors::BLUE),
            list_file_style: style::Style::default().fg(const_colors::DARK_GRAY_BLUE),
            list_directory_style: style::Style::default().fg(const_colors::BLUE).bold(),
            list_highlight_style: style::Style::default().bg(const_colors::LIGHT_GRAY).fg(const_colors::CYAN).bold(),
            list_highlight_symbol: " "
        }
    }
}

pub mod const_colors {
    use super::Color;
    
    pub const CYAN           : Color = Color::Rgb( 25, 225, 235);
    pub const LIGHT_BLUE     : Color = Color::Rgb( 58, 157, 214);
    pub const BLUE           : Color = Color::Rgb(  7, 129, 229);
    pub const GREEN          : Color = Color::Rgb( 21, 194, 146);
    pub const DARK_GREEN     : Color = Color::Rgb( 20, 150, 100);
    pub const GRAY_BLUE      : Color = Color::Rgb(150, 180, 250);
    pub const LIGHT_GRAY     : Color = Color::Rgb( 20,  30,  30);
    pub const DARK_GRAY      : Color = Color::Rgb(120, 120, 120);
    pub const DARK_GRAY_BLUE : Color = Color::Rgb( 90, 110, 150);
}
