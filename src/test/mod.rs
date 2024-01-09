pub mod results;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use serde::{Serialize, Serializer};
use std::fmt;
use std::time::Instant;

pub fn serialize_instant<S>(instant: &Instant, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let duration = instant.elapsed();
    duration.serialize(serializer)
}

#[derive(Clone, Serialize)]
pub struct TestEvent {
    #[serde(
        serialize_with = "serialize_instant",
        deserialize_with = "deserialize_instant"
    )]
    pub time: Instant,
    pub key: KeyEvent,
    pub correct: Option<bool>,
}

impl fmt::Debug for TestEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TestEvent")
            .field("time", &String::from("Instant { ... }"))
            .field("key", &self.key)
            .finish()
    }
}

#[derive(Clone, Debug, Serialize)]
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

impl From<&str> for TestWord {
    fn from(string: &str) -> Self {
        Self::from(string.to_string())
    }
}

#[derive(Debug)]
pub struct Test {
    pub words: Vec<TestWord>,
    pub current_word: usize,
    pub complete: bool,
    pub backtracking_enabled: bool,
}

impl Test {
    pub fn new(words: Vec<String>, backtracking_enabled: bool) -> Self {
        Self {
            words: words.into_iter().map(TestWord::from).collect(),
            current_word: 0,
            complete: false,
            backtracking_enabled,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        let word = &mut self.words[self.current_word];
        match key.code {
            KeyCode::Char(' ') | KeyCode::Enter => {
                if word.text.chars().nth(word.progress.len()) == Some(' ') {
                    word.progress.push(' ');
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(true),
                        key,
                    })
                } else if !word.progress.is_empty() || word.text.is_empty() {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(word.text == word.progress),
                        key,
                    });
                    self.next_word();
                }
            }
            KeyCode::Backspace => {
                if word.progress.is_empty() && self.backtracking_enabled {
                    self.last_word();
                } else {
                    word.events.push(TestEvent {
                        time: Instant::now(),
                        correct: Some(!word.text.starts_with(&word.progress[..])),
                        key,
                    });
                    word.progress.pop();
                }
            }
            // CTRL-BackSpace
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.words[self.current_word].progress.is_empty() {
                    self.last_word();
                }

                let word = &mut self.words[self.current_word];

                word.events.push(TestEvent {
                    time: Instant::now(),
                    correct: None,
                    key,
                });
                word.progress.clear();
            }
            KeyCode::Char(c) => {
                word.progress.push(c);
                word.events.push(TestEvent {
                    time: Instant::now(),
                    correct: Some(word.text.starts_with(&word.progress[..])),
                    key,
                });
                if word.progress == word.text && self.current_word == self.words.len() - 1 {
                    self.complete = true;
                    self.current_word = 0;
                }
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
        } else {
            self.current_word += 1;
        }
    }
}
