use tuirealm::ratatui::crossterm::event::{KeyCode, KeyModifiers};
use tuirealm::{Component, Event, NoUserEvent};

use super::{convert_tuirealm_to_crossterm_key, TestComponent};
use crate::components::result::Results;
use crate::messages::Msg;

impl Component<Msg, NoUserEvent> for TestComponent {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key_event) => {
                // Convert tuirealm KeyEvent to crossterm KeyEvent for Test::handle_key
                let crossterm_key = convert_tuirealm_to_crossterm_key(key_event);

                // Respect the Ctrl-C signal
                if crossterm_key.modifiers == KeyModifiers::CONTROL
                    && crossterm_key.code == KeyCode::Char('c')
                {
                    return Some(Msg::AppClose);
                }

                if crossterm_key.code == KeyCode::Esc {
                    let results = Results::from(&self.test);

                    return Some(Msg::ShowResults(results));
                }

                self.test.handle_key(crossterm_key);
                if self.test.complete {
                    let results = Results::from(&self.test);
                    Some(Msg::ShowResults(results))
                } else {
                    Some(Msg::None)
                }
            }
            // Simply signal that we need a redraw
            Event::WindowResize(_, _) => Some(Msg::None),
            _ => None,
        }
    }
}
