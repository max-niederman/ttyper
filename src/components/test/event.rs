use std::fmt;
use std::time::Instant;
use tuirealm::ratatui::crossterm::event::KeyEvent;

pub struct TestEvent {
    pub time: Instant,
    pub key: KeyEvent,
    pub correct: Option<bool>,
}

pub fn is_missed_word_event(event: &TestEvent) -> bool {
    event.correct != Some(true)
}

impl fmt::Debug for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEvent")
            .field("time", &String::from("Instant { ... }"))
            .field("key", &self.key)
            .finish()
    }
}

impl PartialEq for TestEvent {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.key == other.key && self.correct == other.correct
    }
}

impl Clone for TestEvent {
    fn clone(&self) -> Self {
        Self {
            time: self.time,
            key: self.key,
            correct: self.correct,
        }
    }
}
