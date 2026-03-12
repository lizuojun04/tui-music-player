use crate::{
    app::app::App,
    ui::ui::Drawable,
    utils::utils
};
use ratatui::{
    layout::{Rect},
    text::{Line},
    widgets::{Block, List, ListItem}, 
    Frame
};


pub struct FileBrowserDrawer;

impl Drawable for FileBrowserDrawer {
    fn drawn_ui(frame: &mut Frame, app: &mut App, area: Rect) {
        let inner_area = Self::render_block_with_border(frame, app, area);
        Self::render_file_entry(frame, app, inner_area);
    }
}

impl FileBrowserDrawer {
    fn render_block_with_border(frame: &mut Frame, app: &mut App, area: Rect) -> Rect {
        let block = Block::default()
            .borders(app.theme.file_browser_theme.file_browser_borders)
            .title(Line::from("File Browser").left_aligned())
            .title_bottom(Line::from(utils::format_path_for_display(&app.current_path)).right_aligned())
            .border_type(app.theme.file_browser_theme.file_browser_border_type)
            .border_style(app.theme.file_browser_theme.file_browser_border_style);
        let inner_area = block.inner(area);
        frame.render_widget(block, area);
        inner_area
    }

    fn render_file_entry(frame: &mut Frame, app: &mut App, area: Rect) {
        let list = List::new(
            app.file_browser.items.iter().map(|item| {
                let style = if item.is_file() {
                    app.theme.file_browser_theme.list_file_style
                } else {
                    app.theme.file_browser_theme.list_directory_style
                };
                ListItem::new(item.get_file_name()).style(style)
            })
        )
        .highlight_style(app.theme.file_browser_theme.list_highlight_style)
        .highlight_symbol(format!("{} ", app.theme.file_browser_theme.list_highlight_symbol));
        frame.render_stateful_widget(list, area, &mut app.file_browser_list_state);
    }

}
