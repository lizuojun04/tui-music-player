use crossterm::event::KeyEvent;


pub enum PlayerEvent {
    SongFinished
}

pub enum MainEvent {
    Player(PlayerEvent),
    Key(KeyEvent)
}
