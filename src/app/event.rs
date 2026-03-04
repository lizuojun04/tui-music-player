use crossterm::event::KeyEvent;


pub enum PlayerEvent {
    SongInfo((String, String, String, f64)),
    SongFinished
}

pub enum MainEvent {
    Player(PlayerEvent),
    Key(KeyEvent)
}
