pub mod results;

use std::fmt;
use std::iter::FromIterator;
use std::time::Instant;
use termion::event::Key;

pub struct TestEvent {
    time: Instant,
    correct: bool,
    key: Key,
}

impl fmt::Debug for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimedTestEvent")
            .field("correct", &self.correct)
            .field("key", &self.key)
            .field("time", &String::from("Instant { ... }"))
            .finish()
    }
}

pub struct Test {
    pub events: Vec<TestEvent>,
    pub start: Instant,
    pub complete: bool,
    pub targets: Vec<String>,
    pub current_target: usize,
    pub target_progress: String,
    correct_keys: Vec<Key>,
    // NOTE: In reverse order for O(1) pop and push
    needed_keys: Vec<Key>,
}

impl fmt::Debug for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Test")
            .field("complete", &self.complete)
            .field("current_target", &self.current_target)
            .field("target_progress", &self.target_progress)
            .field("targets", &self.targets)
            .field("events", &String::from_iter(self.events.iter().filter_map(|e| match e.key {
                Key::Backspace => Some(String::from("⌫")),
                Key::Char(c) => Some(c.to_string()),
                _ => None,
            })))
            .field("correct_events", &String::from_iter(self.correct_keys.iter().filter_map(|k| match k {
                Key::Backspace => Some(String::from("⌫")),
                Key::Char(c) => Some(c.to_string()),
                _ => None,
            })))
            .field("needed_events", &String::from_iter(self.needed_keys.iter().rev().filter_map(|k| match k {
                Key::Backspace => Some(String::from("⌫")),
                Key::Char(c) => Some(c.to_string()),
                _ => None,
            })))
            .finish()
    }
}

impl Test {
    pub fn new(targets: Vec<String>) -> Self {
        let mut s = Self {
            events: Vec::new(),
            start: Instant::now(),
            complete: false,
            targets,
            current_target: 0,
            target_progress: String::new(),
            correct_keys: Vec::new(),
            needed_keys: Vec::new(),
        };

        s.needed_keys = s.targets[s.current_target]
            .chars()
            .rev()
            .map(|c| Key::Char(c))
            .collect();
        s
    }

    pub fn handle_key(&mut self, key: Key) {
        let correct = *self.needed_keys.last().unwrap();

        let test_event = match key == correct {
            true => {
                self.needed_keys.pop();
                self.correct_keys.push(key);

                TestEvent {
                    time: Instant::now(),
                    correct: true,
                    key,
                }
            }

            false => {
                match key {
                    Key::Backspace => {
                        if let Some(k) = self.correct_keys.pop() {
                            self.needed_keys.push(k);
                        }
                    }
                    Key::Char(_) => {
                        self.needed_keys.push(Key::Backspace);
                    }
                    _ => {
                        return;
                    }
                };

                TestEvent {
                    time: Instant::now(),
                    correct: false,
                    key,
                }
            }
        };

        self.events.push(test_event);

        match key {
            Key::Backspace => {
                self.target_progress.pop();
            }
            Key::Char(c) => {
                self.target_progress.push(c);
            }
            _ => {}
        }

        if self.needed_keys.is_empty() {
            self.next_target()
        }
    }

    fn next_target(&mut self) {
        self.current_target += 1;
        if self.current_target == self.targets.len() {
            self.current_target = 0;
            self.complete = true;
        }

        self.target_progress.clear();
        self.correct_keys.clear();
        self.needed_keys = self.targets[self.current_target]
            .chars()
            .rev()
            .map(|c| Key::Char(c))
            .collect();
    }
}
