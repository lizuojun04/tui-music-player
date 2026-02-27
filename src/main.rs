mod audio;
mod utils;
mod ui;
mod app;
use cpal::traits::{HostTrait, DeviceTrait};
use audio::player::{Player, PlayerCommand};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph};

use crossterm::event;
use ratatui::{
    layout::{Constraint, Layout},
    widgets::Block,
    Frame,
};

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = app::app::App::new(std::path::PathBuf::from("/home/lizuojun/Music"));
    let res = app.run(&mut terminal);

    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
        return Err(err);
    }

    Ok(())
}
