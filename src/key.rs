use crossterm::event::{KeyCode, KeyModifiers};
use serde::{de, Deserialize};

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct KeyMap {
    #[serde(deserialize_with = "deseralize_key")]
    pub remove_previous_word: Option<Key>,
    #[serde(deserialize_with = "deseralize_key")]
    pub remove_previous_char: Option<Key>,
    #[serde(deserialize_with = "deseralize_key")]
    pub next_word: Option<Key>,
}

fn deseralize_key<'de, D>(deserializer: D) -> Result<Option<Key>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct KeyVisitor;
    impl<'de> de::Visitor<'de> for KeyVisitor {
        type Value = Option<Key>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("key specification")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(get_key_from_string(v))
        }
    }

    return deserializer.deserialize_str(KeyVisitor);
}

#[derive(Debug)]
pub struct Key {
    pub code: KeyCode,
    pub modifier: KeyModifiers,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KeyCode::Null,
            modifier: KeyModifiers::NONE,
        }
    }
}

fn get_key_code_from_string(string: &str) -> KeyCode {
    if string.chars().count() == 1 {
        let key_code_char = string.chars().next();
        if let Some(key_code_char) = key_code_char {
            if key_code_char.is_lowercase() {
                return KeyCode::Char(key_code_char);
            }
        }
    }
    match string {
        "Backspace" => KeyCode::Backspace,
        "Enter" => KeyCode::Enter,
        "Left" => KeyCode::Left,
        "Right" => KeyCode::Right,
        "Up" => KeyCode::Up,
        "Down" => KeyCode::Down,
        "Home" => KeyCode::Home,
        "End" => KeyCode::End,
        "PageUp" => KeyCode::PageUp,
        "PageDown" => KeyCode::PageDown,
        "Tab" => KeyCode::Tab,
        "BackTab" => KeyCode::BackTab,
        "Delete" => KeyCode::Delete,
        "Insert" => KeyCode::Insert,
        "Esc" => KeyCode::Esc,
        "CapsLock" => KeyCode::CapsLock,
        "ScrollLock" => KeyCode::ScrollLock,
        "NumLock" => KeyCode::NumLock,
        "PrintScreen" => KeyCode::PrintScreen,
        "Pause" => KeyCode::Pause,
        "Menu" => KeyCode::Menu,
        "KeypadBegin" => KeyCode::KeypadBegin,
        _ => KeyCode::Null,
    }
}

fn get_key_modifier_from_string(string: &str) -> KeyModifiers {
    match string {
        "C" => KeyModifiers::CONTROL,
        "A" => KeyModifiers::ALT,
        "W" => KeyModifiers::SUPER,
        "H" => KeyModifiers::HYPER,
        "M" => KeyModifiers::META,
        _ => KeyModifiers::NONE,
    }
}

fn get_key_from_string(string: &str) -> Option<Key> {
    let mut key = Key {
        code: KeyCode::Null,
        modifier: KeyModifiers::NONE,
    };
    match string.split('-').count() {
        1 => {
            if string.chars().count() == 1 {
                key.code = KeyCode::Null;
            } else {
                key.code = get_key_code_from_string(string);
            }
        }
        2 => {
            let mut split = string.split('-');
            let key_code = split.next();
            if let Some(key_code) = key_code {
                if key_code.chars().count() == 1 {
                    key.modifier = get_key_modifier_from_string(key_code);
                }
            }
            if key.modifier != KeyModifiers::NONE {
                let key_code = split.next();
                if let Some(key_code) = key_code {
                    key.code = get_key_code_from_string(key_code);
                    if key.code == KeyCode::Null {
                        key.modifier = KeyModifiers::NONE;
                    }
                }
            }
        }
        _ => {}
    }
    if key.modifier == KeyModifiers::NONE && key.code == KeyCode::Null {
        return None;
    }
    Some(key)
}
