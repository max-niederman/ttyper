use rand::{seq::SliceRandom, thread_rng};
use tuirealm::{
    event::{Key, KeyModifiers},
    Component, Event, NoUserEvent,
};

use super::ResultsComponent;
use crate::messages::Msg;

impl Component<Msg, NoUserEvent> for ResultsComponent {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key)
                if key.code == Key::Char('q') && key.modifiers == KeyModifiers::NONE =>
            {
                Some(Msg::AppClose)
            }

            Event::Keyboard(key)
                if key.code == Key::Char('r') && key.modifiers == KeyModifiers::NONE =>
            {
                Some(Msg::RestartTest)
            }

            Event::Keyboard(key)
                if key.code == Key::Char('p') && key.modifiers == KeyModifiers::NONE =>
            {
                if self.results.missed_words.is_empty() {
                    return None;
                }
                // repeat each missed word 5 times
                let mut practice_words: Vec<String> = (self.results.missed_words)
                    .iter()
                    .flat_map(|w: &String| vec![w.clone(); 5])
                    .collect();

                practice_words.shuffle(&mut thread_rng());

                Some(Msg::StartTest(practice_words))
            }
            _ => None,
        }
    }
}
