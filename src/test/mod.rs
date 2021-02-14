pub mod results;

use std::fmt;
use std::time::Instant;
use termion::event::Key;

pub struct TestEvent {
    time: Instant,
    key: Key,
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
    pub word: String,
    pub events: Vec<TestEvent>,
}

impl TestWord {
    fn entered_string(&self) -> String {
        let mut s = String::new();
        for e in &self.events {
            match e.key {
                Key::Backspace => {
                    s.pop();
                },
                Key::Char(c) => {
                    s.push(c);
                },
                _ => {},
            }
        }
        s
    }
}

impl From<String> for TestWord {
    fn from(word: String) -> Self {
        TestWord {
            word,
            events: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Test {
    pub words: Vec<TestWord>,
    pub word_progress: String,
    current_word: usize,
    pub complete: bool,
}

impl Test {
    pub fn new(words: Vec<String>) -> Self {
        Self {
            words: words.into_iter().map(|w| TestWord::from(w)).collect(),
            word_progress: String::new(),
            current_word: 0,
            complete: false,
        }
    }

    pub fn handle_key(&mut self, key: Key) {
        let word = self.words.get_mut(self.current_word).unwrap();

        match key {
            Key::Char(' ') | Key::Char('\n') => match self.word_progress.len() {
                0 => {}
                _ => self.next_word(),
            },
            Key::Backspace => match self.word_progress.len() {
                0 => self.last_word(),
                _ => {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        key,
                    });
                    self.word_progress.pop();
                }
            },
            Key::Char(c) => {
                word.events.push(TestEvent {
                    time: Instant::now(),
                    key,
                });
                self.word_progress.push(c);
            },
            _ => {},
        };
    }

    fn last_word(&mut self) {
        if self.current_word == 0 {
            return;
        }

        self.current_word -= 1;
        self.word_progress = self.words[self.current_word].entered_string();
    }

    fn next_word(&mut self) {
        self.word_progress.clear();

        if self.current_word == self.words.len() {
            self.complete = true;
            self.current_word = 0;
            return;
        }

        self.current_word += 1;
    }
}
