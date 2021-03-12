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

impl From<&Test> for Results {
    fn from(test: &Test) -> Self {
        Self {
            cps: {
                let mut cps = CPSData {
                    overall: -1f64,
                    per_event: Vec::new(),
                    per_key: [-1f64; 256],
                };

                let mut events = test.words.iter().flat_map(|w| w.events.iter());

                let mut word_freq = [0usize; 256];
                for e in events.clone().skip(1) {
                    match e.key {
                        Key::Backspace => word_freq[AsciiChar::BackSpace as usize] += 1,
                        Key::Char(c) => AsciiChar::from_ascii(c)
                            .map(|ac| word_freq[ac as usize] += 1)
                            .unwrap_or(()),
                        _ => {}
                    }
                }

                let mut last = events
                    .next()
                    .expect("Error while calculating results.")
                    .time;
                for e in events {
                    let event_cps = (e.time - last).as_secs_f64().recip();
                    cps.per_event.push(event_cps);

                    match e.key {
                        Key::Backspace => {
                            cps.per_key[AsciiChar::BackSpace as usize] +=
                                event_cps / word_freq[AsciiChar::BackSpace as usize] as f64
                        }
                        Key::Char(c) => AsciiChar::from_ascii(c)
                            .map(|ac| {
                                cps.per_key[ac as usize] +=
                                    event_cps / word_freq[ac as usize] as f64
                            })
                            .unwrap_or(()),
                        _ => {}
                    }

                    last = e.time;
                }

                cps.overall =
                    cps.per_event.iter().fold(0f64, |acc, c| acc + c) / cps.per_event.len() as f64;

                cps
            },
            accuracy: {
                let mut acc = AccuracyData {
                    overall: Fraction::default(),
                    per_key: [Fraction::default(); 256],
                };

                fn increment(acc: &mut AccuracyData, ch: AsciiChar, cr: bool) {
                    acc.overall.denominator += 1;
                    acc.per_key[ch as usize].denominator += 1;
                    if cr {
                        acc.overall.numerator += 1;
                        acc.per_key[ch as usize].numerator += 1;
                    }
                }

                let mut progress = String::new();
                for word in test.words.iter().filter(|w| !w.events.is_empty()) {
                    progress.clear();
                    for event in word.events.iter() {
                        match event.key {
                            Key::Backspace => {
                                increment(
                                    &mut acc,
                                    AsciiChar::BackSpace,
                                    !word.text.starts_with(&progress),
                                );
                                progress.pop();
                            }
                            Key::Char(c) => {
                                progress.push(c);
                                if let Some(ac) = AsciiChar::from_ascii(c).ok() {
                                    increment(&mut acc, ac, word.text.starts_with(&progress));
                                }
                            }
                            _ => {}
                        }
                    }
                }

                acc
            },
        }
    }
}
