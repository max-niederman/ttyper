use tuirealm::event::Key;
use tuirealm::ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
};

use crate::config::Theme;

pub mod component;
pub mod event;
pub mod handler;
pub mod mock_component;
pub mod spans;
pub mod status;
pub mod word;

pub use word::TestWord;

pub struct TestComponent {
    pub test: Test,
    pub theme: Theme,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub words: Vec<TestWord>,
    pub current_word: usize,
    pub complete: bool,
    pub backtracking_enabled: bool,
    pub sudden_death_enabled: bool,
    pub backspace_enabled: bool,
}

impl Test {
    pub fn new(
        words: Vec<String>,
        backtracking_enabled: bool,
        sudden_death_enabled: bool,
        backspace_enabled: bool,
    ) -> Self {
        Self {
            words: words.into_iter().map(TestWord::from).collect(),
            current_word: 0,
            complete: false,
            backtracking_enabled,
            sudden_death_enabled,
            backspace_enabled,
        }
    }
}

fn convert_tuirealm_to_crossterm_key(
    key: tuirealm::event::KeyEvent,
) -> tuirealm::ratatui::crossterm::event::KeyEvent {
    let code = match key.code {
        Key::Backspace => KeyCode::Backspace,
        Key::Enter => KeyCode::Enter,
        Key::Left => KeyCode::Left,
        Key::Right => KeyCode::Right,
        Key::Up => KeyCode::Up,
        Key::Down => KeyCode::Down,
        Key::Home => KeyCode::Home,
        Key::End => KeyCode::End,
        Key::PageUp => KeyCode::PageUp,
        Key::PageDown => KeyCode::PageDown,
        Key::Tab => KeyCode::Tab,
        Key::BackTab => KeyCode::BackTab,
        Key::Delete => KeyCode::Delete,
        Key::Insert => KeyCode::Insert,
        Key::Function(n) => KeyCode::F(n),
        Key::Char(c) => KeyCode::Char(c),
        Key::Null => KeyCode::Null,
        Key::Esc => KeyCode::Esc,
        Key::CapsLock => KeyCode::CapsLock,
        Key::ScrollLock => KeyCode::ScrollLock,
        Key::NumLock => KeyCode::NumLock,
        Key::PrintScreen => KeyCode::PrintScreen,
        Key::Pause => KeyCode::Pause,
        Key::Menu => KeyCode::Menu,
        Key::KeypadBegin => KeyCode::KeypadBegin,
        _ => KeyCode::Null,
    };

    let mut modifiers = KeyModifiers::empty();
    if key.modifiers.contains(tuirealm::event::KeyModifiers::SHIFT) {
        modifiers.insert(KeyModifiers::SHIFT);
    }
    if key
        .modifiers
        .contains(tuirealm::event::KeyModifiers::CONTROL)
    {
        modifiers.insert(KeyModifiers::CONTROL);
    }
    if key.modifiers.contains(tuirealm::event::KeyModifiers::ALT) {
        modifiers.insert(KeyModifiers::ALT);
    }

    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    }
}
