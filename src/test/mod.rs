pub mod results;

use std::fmt;
use std::time::Instant;
use termion::event::Key;

pub struct TestEvent {
    pub time: Instant,
    pub key: Key,
}

impl fmt::Debug for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEvent")
            .field("time", &String::from("Instant { ... }"))
            .field("key", &self.key)
            .finish()
    }
}

#[derive(Debug)]
pub struct TestWord {
    pub text: String,
    pub progress: String,
    pub events: Vec<TestEvent>,
}

impl From<String> for TestWord {
    fn from(string: String) -> Self {
        TestWord {
            text: string,
            progress: String::new(),
            events: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Test {
    pub words: Vec<TestWord>,
    pub current_word: usize,
    pub complete: bool,
}

impl Test {
    pub fn new(words: Vec<&String>) -> Self {
        Self {
            words: words
                .into_iter()
                .map(|w| TestWord::from(w.clone()))
                .collect(),
            current_word: 0,
            complete: false,
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        let word = self.words.get_mut(self.current_word).unwrap();
        
        match key {
            Key::Char(' ') | Key::Char('\n') => {
                if !word.progress.is_empty() {
                    self.next_word();
                }
            }
            Key::Backspace => match word.progress.len() {
                0 => self.last_word(),
                _ => {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        key,
                    });
                    word.progress.pop();
                }
            },
            // At least on Linux, Ctrl-Backspace is mapped to Ctrl('h')
            Key::Ctrl('\x08') | Key::Ctrl('h') => {
                word.events.push(TestEvent {
                    time: Instant::now(),
                    key,
                });
                word.progress.clear();
            },
            Key::Char(c) => {
                word.events.push(TestEvent {
                    time: Instant::now(),
                    key,
                });
                word.progress.push(c);
            }
            _ => {}
        };
    }

    fn last_word(&mut self) {
        if self.current_word != 0 {
            self.current_word -= 1;
        }
    }

    fn next_word(&mut self) {
        if self.current_word == self.words.len() - 1 {
            self.complete = true;
            self.current_word = 0;
            return;
        }

        self.current_word += 1;
    }
}
