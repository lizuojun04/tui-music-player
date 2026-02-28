use crate::app::event::MainEvent;

pub struct KeyInput;

impl KeyInput {
    pub fn listen_key_input(event_sender: crossbeam_channel::Sender<MainEvent>) {
        std::thread::spawn(move || loop {
            if let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read() 
                && event_sender.send(MainEvent::Key(key)).is_err() {
                    break;
            }
        });
    }
}
