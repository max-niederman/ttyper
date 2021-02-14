use super::Test;

use std::fmt;
use std::time::Instant;
use termion::event::Key;

pub struct Fraction {
    numerator: usize,
    denominator: usize,
}

impl From<Fraction> for f32 {
    fn from(f: Fraction) -> Self {
        match f.denominator {
            0 => 1 as f32,
            _ => (f.numerator / f.denominator) as f32,
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
    fn wpm(&self) -> f32;
    fn accuracy(&self) -> Fraction;
}

impl PartialResults for Test {
    fn progress(&self) -> Fraction {
        let total: usize = self.targets.iter().map(|t| t.len()).sum();
        let done = self
            .events
            .iter()
            .filter(|event| event.correct && event.key != Key::Backspace)
            .count();

        Fraction {
            numerator: done,
            denominator: total,
        }
    }

    fn wpm(&self) -> f32 {
        match self.events.get(0) {
            Some(e) => {
                let chars = self.progress().numerator;
                let timer = Instant::now() - e.time;
                (chars as f32 / 5.0) / (timer.as_secs_f32() / 60.0)
            }
            None => 0 as f32,
        }
    }

    fn accuracy(&self) -> Fraction {
        let total: usize = self.events.iter().count();
        let correct: usize = self.events.iter().filter(|event| event.correct).count();
        Fraction {
            numerator: correct,
            denominator: total,
        }
    }
}
