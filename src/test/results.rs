use super::Test;

use std::fmt;

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
}

impl PartialResults for Test {
    fn progress(&self) -> Fraction {
        Fraction {
            numerator: self.current_word + 1,
            denominator: self.words.len(),
        }
    }
}
