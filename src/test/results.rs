use super::Test;

use crossterm::event::KeyEvent;
use std::collections::HashMap;
use std::{cmp, fmt};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fraction {
    pub numerator: usize,
    pub denominator: usize,
}

impl Fraction {
    pub const fn new(numerator: usize, denominator: usize) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl From<Fraction> for f64 {
    fn from(f: Fraction) -> Self {
        f.numerator as f64 / f.denominator as f64
    }
}

impl cmp::Ord for Fraction {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        f64::from(*self).partial_cmp(&f64::from(*other)).unwrap()
    }
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
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

pub struct CpsData {
    pub overall: f64,
    pub per_event: Vec<f64>,
    pub per_key: HashMap<KeyEvent, f64>,
}

pub struct AccuracyData {
    pub overall: Fraction,
    pub per_key: HashMap<KeyEvent, Fraction>,
}

pub struct Results {
    pub cps: CpsData,
    pub accuracy: AccuracyData,
}

impl From<Test> for Results {
    fn from(test: Test) -> Self {
        let events: Vec<&super::TestEvent> =
            test.words.iter().flat_map(|w| w.events.iter()).collect();
        Self {
            cps: {
                let mut cps = CpsData {
                    overall: 0f64,
                    per_event: Vec::new(),
                    per_key: HashMap::new(),
                };

                let mut keys: HashMap<KeyEvent, (f64, usize)> = HashMap::new();

                // NOTE: this should really be optimized to use less than O(n) space
                for win in events.windows(2) {
                    let event_cps = win[1]
                        .time
                        .checked_duration_since(win[0].time)
                        .map(|d| d.as_secs_f64().recip());

                    if let Some(event_cps) = event_cps {
                        cps.per_event.push(event_cps);

                        let key = keys.entry(win[1].key).or_insert((0.0, 0));
                        key.0 += event_cps;
                        key.1 += 1;
                    }
                }

                cps.per_key = keys
                    .into_iter()
                    .map(|(key, (total, count))| (key, total / count as f64))
                    .collect();

                cps.overall = cps.per_event.iter().sum::<f64>() / cps.per_event.len() as f64;

                cps
            },
            accuracy: {
                let mut acc = AccuracyData {
                    overall: Fraction::new(0, 0),
                    per_key: HashMap::new(),
                };

                events
                    .iter()
                    .filter(|event| event.correct.is_some())
                    .for_each(|event| {
                        let key = acc.per_key.entry(event.key).or_insert(Fraction::new(0, 0));

                        acc.overall.denominator += 1;
                        key.denominator += 1;

                        if event.correct.unwrap() {
                            acc.overall.numerator += 1;
                            key.numerator += 1;
                        }
                    });

                acc
            },
        }
    }
}
