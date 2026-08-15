use crate::app::event::MainEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

pub struct KeyInput;

impl KeyInput {
    pub fn listen_key_input(
        event_sender: crossbeam_channel::Sender<MainEvent>,
        enabled: Arc<AtomicBool>,
    ) {
        thread::spawn(move || {
            loop {
                if !enabled.load(Ordering::Relaxed) {
                    thread::sleep(Duration::from_millis(50));
                    continue;
                }

                let Ok(true) = crossterm::event::poll(Duration::from_millis(100)) else {
                    continue;
                };
                if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() {
                    if key.code == KeyCode::Char('z')
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        enabled.store(false, Ordering::Relaxed);
                    }
                    if event_sender.send(MainEvent::Key(key)).is_err() {
                        break;
                    }
                }
            }
        });
    }
}
