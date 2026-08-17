mod app;
mod audio;
mod protocol;
mod ui;
mod utils;

use crate::{
    app::app::{App, AppExit},
    audio::{client::PlayerClient, daemon::PlayerDaemon},
    protocol::PlayerRequest,
    utils::key_input::KeyInput,
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    env,
    io::{self, stdout},
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("tui-music-player: {error}");
        std::process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let command = env::args().nth(1);
    match command.as_deref() {
        None => {
            ensure_daemon()?;
            run_tui(None)
        }
        Some("root-dir")       => {
            let root_dir_arg = env::args().nth(2).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "missing argument for 'root_dir' command",
                )
            })?;
            let root_dir = PathBuf::from(root_dir_arg);
            ensure_daemon()?;
            run_tui(Some(root_dir))
        }
        Some("daemon")         => PlayerDaemon::run(),
        Some("toggle-play")    => {
            if PlayerClient.state()?.is_playing {
                send_command(PlayerRequest::Pause)
            } else {
                send_command(PlayerRequest::Continue)
            }
        }
        Some("stop")           => send_command(PlayerRequest::Shutdown),
        Some("next")           => send_command(PlayerRequest::Next),
        Some("prev")           => send_command(PlayerRequest::Prev),
        Some("toggle-shuffle") => {
            send_command(PlayerRequest::SetShuffle { enabled: !PlayerClient.state()?.shuffle })
        }
        Some(other)      => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "unknown command '{other}'\nusage: tui-music-player [daemon|toggle-play|stop|next|prev|toggle-shuffle]"
            ),
        )),
    }
}

fn ensure_daemon() -> io::Result<()> {
    let client = PlayerClient;
    if client.is_running() {
        return Ok(());
    }

    let executable = env::current_exe()?;
    let mut command = Command::new(executable);
    command
        .arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command.spawn()?;

    let deadline = Instant::now() + Duration::from_secs(3);
    while Instant::now() < deadline {
        if client.is_running() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }
    Err(io::Error::new(
        io::ErrorKind::TimedOut,
        "daemon did not become ready within 3 seconds; run `tui-music-player daemon` to inspect the error",
    ))
}

fn send_command(request: PlayerRequest) -> io::Result<()> {
    PlayerClient.request(&request).map(|_| ()).map_err(|error| {
        let message = match error.kind() {
            io::ErrorKind::NotFound
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset => format!(
                "cannot contact the daemon: {error}; start it with `tui-music-player daemon`"
            ),
            _ => format!("daemon command failed: {error}"),
        };
        io::Error::new(error.kind(), message)
    })
}

fn run_tui(root_dir: Option<PathBuf>) -> io::Result<()> {
    let _root_dir = match root_dir {
        Some(dir) => dir,
        None => env::current_dir().unwrap_or_else(|_| PathBuf::from("/home/")),
    };
    let mut app = App::new(_root_dir);
    let input_enabled = Arc::new(AtomicBool::new(false));
    KeyInput::listen_key_input(app.event_sender.clone(), input_enabled.clone());

    loop {
        wait_until_foreground();
        crossterm::terminal::enable_raw_mode()?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::EnterAlternateScreen
        )?;
        input_enabled.store(true, Ordering::Relaxed);

        let result = app.run(&mut terminal);
        input_enabled.store(false, Ordering::Relaxed);
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        match result? {
            AppExit::Quit => return Ok(()),
            AppExit::Suspend => suspend_tui(),
        }
    }
}

fn suspend_tui() {
    // SAFETY: raising SIGTSTP for the current process is equivalent to the
    // terminal driver's normal Ctrl-Z handling. The terminal is restored first.
    unsafe {
        libc::raise(libc::SIGTSTP);
    }
}

fn wait_until_foreground() {
    // `bg` resumes the TUI process, but background jobs must not read from the
    // terminal. Wait quietly until `fg` makes this process group foreground.
    unsafe {
        if libc::isatty(libc::STDIN_FILENO) != 1 {
            return;
        }
        while libc::tcgetpgrp(libc::STDIN_FILENO) != libc::getpgrp() {
            thread::sleep(Duration::from_millis(100));
        }
    }
}

