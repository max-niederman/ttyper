use crate::calculate;
use crate::components::test::event::TestEvent;
use crate::components::test::Test;
use crate::config::Theme;

pub mod accuracy;
pub mod component;
pub mod fraction;
pub mod mock_component;
pub mod timing;

pub use accuracy::Data as AccuracyData;
pub use fraction::Fraction;
pub use timing::Data as TimingData;

// Convert CPS to WPM (clicks per second)
pub const WORDS_PER_MINUTE_PER_CPS: f64 = 12.0;

// Width of the moving average window for the WPM chart
pub const WORDS_PER_MINUTE_MOVING_AVERAGE_WIDTH: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct Results {
    pub timing: TimingData,
    pub accuracy: AccuracyData,
    pub missed_words: Vec<String>,
}

impl From<&Test> for Results {
    fn from(test: &Test) -> Self {
        let events: Vec<&TestEvent> = test.words.iter().flat_map(|w| w.events.iter()).collect();

        Self {
            timing: calculate::timing(&events),
            accuracy: calculate::accuracy(&events),
            missed_words: calculate::missed_words(test),
        }
    }
}

pub struct ResultsComponent {
    pub results: Results,
    pub theme: Theme,
}
