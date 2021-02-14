pub mod results;

use std::fmt;
use std::iter::FromIterator;
use std::time::Instant;
use termion::event::Key;

#[derive(Clone, Copy, Debug)]
pub struct TestEvent {
    key: Key,
    correct: bool,
}

pub struct TimedTestEvent {
    time: Instant,
    event: TestEvent,
}

impl fmt::Debug for TimedTestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimedTestEvent")
            .field("time", &String::from("Instant { ... }"))
            .field("event", &self.event)
            .finish()
    }
}

impl From<TestEvent> for TimedTestEvent {
    fn from(event: TestEvent) -> Self {
        Self {
            event,
            time: Instant::now(),
        }
    }
}

pub struct Test {
    pub events: Vec<TimedTestEvent>,
    pub start: Instant,
    pub complete: bool,
    pub targets: Vec<String>,
    pub current_target: usize,
    pub target_progress: String,
    correct_events: Vec<TestEvent>,
    // NOTE: In reverse order for O(1) pop and push
    needed_events: Vec<TestEvent>,
}

impl fmt::Debug for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Test")
            .field("complete", &self.complete)
            .field("current_target", &self.current_target)
            .field("target_progress", &self.target_progress)
            .field("targets", &self.targets)
            .field("events", &String::from_iter(self.events.iter().filter_map(|e| match e.event.key {
                Key::Backspace => Some(String::from("⌫")),
                Key::Char(c) => Some(c.to_string()),
                _ => None,
            })))
            .field("correct_events", &String::from_iter(self.correct_events.iter().filter_map(|e| match e.key {
                Key::Backspace => Some(String::from("⌫")),
                Key::Char(c) => Some(c.to_string()),
                _ => None,
            })))
            .field("needed_events", &String::from_iter(self.needed_events.iter().rev().filter_map(|e| match e.key {
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
            correct_events: Vec::new(),
            needed_events: Vec::new(),
        };

        s.needed_events = s.targets[s.current_target]
            .chars()
            .rev()
            .map(|c| TestEvent {
                key: Key::Char(c),
                correct: true,
            })
            .collect();
        s
    }

    pub fn handle_key(&mut self, key: Key) {
        let correct = self.needed_events.last().unwrap().key;

        let timed_event = match key == correct {
            true => {
                let event = TestEvent { correct: true, key };

                self.needed_events.pop();
                self.correct_events.push(event);

                TimedTestEvent::from(event)
            }

            false => {
                let event = TestEvent {
                    correct: false,
                    key,
                };

                match key {
                    Key::Backspace => {
                        if let Some(e) = self.correct_events.pop() {
                            self.needed_events.push(e);
                        }
                    }
                    Key::Char(_) => {
                        self.needed_events.push(TestEvent {
                            correct: true,
                            key: Key::Backspace,
                        });
                    }
                    _ => {
                        return;
                    }
                };

                TimedTestEvent::from(event)            }
        };

        self.events.push(timed_event);

        match key {
            Key::Backspace => {
                self.target_progress.pop();
            }
            Key::Char(c) => {
                self.target_progress.push(c);
            }
            _ => {}
        }

        if self.needed_events.is_empty() {
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
        self.correct_events.clear();
        self.needed_events = self.targets[self.current_target]
            .chars()
            .rev()
            .map(|c| TestEvent {
                key: Key::Char(c),
                correct: true,
            })
            .collect();
    }
}
