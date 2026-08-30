use crate::components::result::accuracy::Data as AccuracyData;
use crate::components::result::fraction::Fraction;
use crate::components::result::timing::Data as TimingData;
use crate::components::test::event::{is_missed_word_event, TestEvent};
use crate::components::test::Test;
use std::collections::HashMap;
use tuirealm::ratatui::crossterm::event::KeyEvent;

pub fn timing(events: &[&TestEvent]) -> TimingData {
    let mut timing = TimingData {
        overall_cps: -1.0,
        per_event: Vec::new(),
        per_key: HashMap::new(),
    };

    // map of keys to a two-tuple (total time, clicks) for counting average
    let mut keys: HashMap<KeyEvent, (f64, usize)> = HashMap::new();

    for win in events.windows(2) {
        let event_dur = win[1]
            .time
            .checked_duration_since(win[0].time)
            .map(|d| d.as_secs_f64());

        if let Some(event_dur) = event_dur {
            timing.per_event.push(event_dur);

            let key = keys.entry(win[1].key).or_insert((0.0, 0));
            key.0 += event_dur;
            key.1 += 1;
        }
    }

    timing.per_key = keys
        .into_iter()
        .map(|(key, (total, count))| (key, total / count as f64))
        .collect();

    timing.overall_cps = if timing.per_event.is_empty() {
        0.0
    } else {
        timing.per_event.len() as f64 / timing.per_event.iter().sum::<f64>()
    };

    timing
}

pub fn accuracy(events: &[&TestEvent]) -> AccuracyData {
    let mut acc = AccuracyData {
        overall: Fraction::new(0, 0),
        per_key: HashMap::new(),
    };

    events
        .iter()
        .filter(|event| event.correct.is_some())
        .for_each(|event| {
            let key = acc
                .per_key
                .entry(event.key)
                .or_insert_with(|| Fraction::new(0, 0));

            acc.overall.denominator += 1;
            key.denominator += 1;

            if event.correct.unwrap_or(false) {
                acc.overall.numerator += 1;
                key.numerator += 1;
            }
        });

    acc
}

pub fn missed_words(test: &Test) -> Vec<String> {
    test.words
        .iter()
        .filter(|word| word.events.iter().any(is_missed_word_event))
        .map(|word| word.text.clone())
        .collect()
}
