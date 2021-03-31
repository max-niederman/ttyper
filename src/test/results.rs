use super::Test;

use ascii::AsciiChar;
use crossterm::event::{KeyCode, KeyEvent};
use std::convert::TryInto;
use std::fmt;
use std::num::NonZeroUsize;

#[derive(Clone, Copy, Debug)]
pub struct Fraction {
    numerator: usize,
    denominator: NonZeroUsize,
}

impl Fraction {
    fn default() -> Self {
        Self {
            numerator: 0,
            denominator: NonZeroUsize::new(1).unwrap(),
        }
    }
}

impl From<Fraction> for f64 {
    fn from(f: Fraction) -> Self {
        f.numerator as f64 / f.denominator.get() as f64
    }
}

impl fmt::Display for Fraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.numerator, self.denominator)
    }
}

pub trait PartialResults {
    fn progress(&self) -> Fraction;
}

impl PartialResults for Test {
    fn progress(&self) -> Fraction {
        Fraction {
            numerator: self.current_word + 1,
            denominator: self
                .words
                .len()
                .try_into()
                .unwrap_or_else(|_| NonZeroUsize::new(1).unwrap()),
        }
    }
}

pub struct CpsData {
    pub overall: f64,
    pub per_event: Vec<f64>,
    pub per_key: [f64; 256],
}

pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: [Fraction; 256],
}

pub struct Results {
    pub cps: CpsData,
    pub accuracy: AccuracyData,
}

trait FromTermKey {
    fn from_key(key: KeyEvent) -> Self;
}
impl FromTermKey for Option<AsciiChar> {
    fn from_key(key: KeyEvent) -> Self {
        match key.code {
            KeyCode::Backspace => Some(AsciiChar::BackSpace),
            KeyCode::Delete => Some(AsciiChar::DEL),
            KeyCode::Char(c) => AsciiChar::from_ascii(c).ok(),
            KeyCode::Null => Some(AsciiChar::Null),
            KeyCode::Esc => Some(AsciiChar::ESC),
            _ => None,
        }
    }
}

impl From<Test> for Results {
    fn from(test: Test) -> Self {
        let events = test.words.iter().flat_map(|w| w.events.iter());
        Self {
            cps: {
                let mut cps = CpsData {
                    overall: 0f64,
                    per_event: Vec::new(),
                    per_key: [0f64; 256],
                };

                let mut key_count = [0usize; 256];

                // NOTE: this should really be optimized to use less than O(n) space
                let event_vec: Vec<&super::TestEvent> = events.clone().collect();
                for win in event_vec.windows(2) {
                    let event_cps = win[1]
                        .time
                        .checked_duration_since(win[0].time)
                        .map(|d| d.as_secs_f64().recip());

                    if let Some(event_cps) = event_cps {
                        cps.per_event.push(event_cps);

                        if let Some(ac) = Option::<AsciiChar>::from_key(win[1].key) {
                            cps.per_key[ac as usize] += event_cps;
                            key_count[ac as usize] += 1;
                        }
                    }
                }
                cps.per_key
                    .iter_mut()
                    .zip(key_count.iter())
                    .for_each(|(key, count)| *key /= *count as f64);

                cps.overall =
                    cps.per_event.iter().fold(0f64, |acc, c| acc + c) / cps.per_event.len() as f64;

                cps
            },
            accuracy: {
                let mut acc = AccuracyData {
                    overall: Fraction::default(),
                    per_key: [Fraction::default(); 256],
                };

                events
                    .filter(|event| event.correct.is_some())
                    .for_each(|event| {
                        if let Some(ch) = Option::<AsciiChar>::from_key(event.key) {
                            acc.overall.denominator =
                                (acc.overall.denominator.get() + 1).try_into().unwrap();
                            acc.per_key[ch as usize].denominator =
                                (acc.overall.denominator.get() + 1).try_into().unwrap();

                            if event.correct.unwrap() {
                                acc.overall.numerator += 1;
                                acc.per_key[ch as usize].numerator += 1;
                            }
                        }
                    });

                acc
            },
        }
    }
}
