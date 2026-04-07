use std::collections::HashMap;
use tuirealm::ratatui::crossterm::event::KeyEvent;

#[derive(Clone, Debug, PartialEq)]
pub struct Data {
    // Instead of storing WPM, we store CPS (clicks per second)
    pub overall_cps: f64,
    pub per_event: Vec<f64>,
    pub per_key: HashMap<KeyEvent, f64>,
}
