use super::Test;

use ascii::AsciiChar;
use std::fmt;
use termion::event::Key;

#[derive(Clone, Copy, Debug)]
pub struct Fraction {
    numerator: usize,
    denominator: usize,
}

impl Fraction {
    fn default() -> Self {
        Self {
            numerator: 0,
            denominator: 0,
        }
    }
}

impl From<Fraction> for f64 {
    fn from(f: Fraction) -> Self {
        match f.denominator {
            0 => 1 as f64,
            _ => f.numerator as f64 / f.denominator as f64,
        }
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
            denominator: self.words.len(),
        }
    }
}

#[derive(Debug)]
pub struct CPSData {
    pub overall: f64,
    pub per_event: Vec<f64>,
    pub per_key: [f64; 256],
}

#[derive(Debug)]
pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: [Fraction; 256],
}

#[derive(Debug)]
pub struct Results {
    pub cps: CPSData,
    pub accuracy: AccuracyData,
}

trait FromTermionKey {
    fn from_key(key: Key) -> Self;
}
impl FromTermionKey for Option<AsciiChar> {
    fn from_key(key: Key) -> Self {
        match key {
            Key::Backspace => Some(AsciiChar::BackSpace),
            Key::Delete => Some(AsciiChar::DEL),
            Key::Char(c) => AsciiChar::from_ascii(c).ok(),
            Key::Null => Some(AsciiChar::Null),
            Key::Esc => Some(AsciiChar::ESC),
            _ => None,
        }
    }
}

impl From<&Test> for Results {
    fn from(test: &Test) -> Self {
        Self {
            cps: {
                let mut cps = CPSData {
                    overall: 0f64,
                    per_event: Vec::new(),
                    per_key: [0f64; 256],
                };

                let mut events = test.words.iter().flat_map(|w| w.events.iter());

                let mut last = events.next().expect("Error while calculating results.");
                let mut key_count = [0usize; 256];
                for event in events.clone() {
                    let event_cps =
                        std::panic::catch_unwind(|| (event.time - last.time).as_secs_f64().recip())
                            .map_err(|_| {
                                println!("Last Word: {:?}", test.words[test.words.len() - 1]);
                                println!("Current Event: {:?} at {:?}", event, event.time);
                                println!("Last Event: {:?} at {:?}", last, last.time);
                            })
                            .unwrap();
                    cps.per_event.push(event_cps);

                    Option::<AsciiChar>::from_key(event.key).map(|ac| {
                        cps.per_key[ac as usize] += event_cps;
                        key_count[ac as usize] += 1;
                    });

                    last = &event;
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

                test.words
                    .iter()
                    .flat_map(|w| w.events.iter())
                    .filter(|event| event.correct.is_some())
                    .for_each(|event| {
                        if let Some(ch) = Option::<AsciiChar>::from_key(event.key) {
                            acc.overall.denominator += 1;
                            acc.per_key[ch as usize].denominator += 1;
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
