use super::Test;

use std::time::Instant;
use termion::event::Key;

pub trait PartialResults {
    fn progress(&self) -> (usize, usize);
    fn wpm(&self) -> f32;
    fn accuracy(&self) -> (usize, usize);
}

impl PartialResults for Test {
    fn progress(&self) -> (usize, usize) {
        let total: usize = self.targets.iter().map(|t| t.len()).sum();
        let done = self.events.iter().filter(|event| event.correct && event.key != Key::Backspace).count();
        (done, total)
    }

    fn wpm(&self) -> f32 {
        let chars = self.progress().0;
        let timer = Instant::now() - self.start;
        (chars as f32 / 5.0) / (timer.as_secs_f32() / 60.0)
    }

    fn accuracy(&self) -> (usize, usize) {
        let total: usize = self.events.iter().count();
        let correct: usize = self.events.iter().filter(|event| event.correct).count();
        (correct, total)
    }
}
