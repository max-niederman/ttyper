use crossterm::event::{KeyCode, KeyModifiers};
use serde::{de, Deserialize};

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct KeyMap {
    #[serde(deserialize_with = "deseralize_key")]
    pub remove_previous_word: Option<Key>,
}

fn deseralize_key<'de, D>(deserializer: D) -> Result<Option<Key>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct KeyVisitor;
    impl<'de> de::Visitor<'de> for KeyVisitor {
        type Value = Option<Key>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("")
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

fn get_key_from_string(v: &str) -> Option<Key> {
    let mut key = Key {
        code: KeyCode::Null,
        modifier: KeyModifiers::NONE,
    };
    match v.split('-').count() {
        2 => {
            let mut split = v.split('-');
            let key_code = split.next();
            if let Some(key_code) = key_code {
                if key_code.len() == 1 {
                    let key_code_char = key_code.chars().next();
                    if let Some(key_code_char) = key_code_char {
                        match key_code_char {
                            'C' => {
                                key.modifier = KeyModifiers::CONTROL;
                            }
                            'A' => {
                                key.modifier = KeyModifiers::ALT;
                            }
                            _ => {}
                        }
                    }
                }
            }
            let key_code = split.next();
            if let Some(key_code) = key_code {
                if key_code.len() == 1 {
                    let key_code_char = key_code.chars().next();
                    if let Some(key_code_char) = key_code_char {
                        if key_code_char.is_lowercase() {
                            key.code = KeyCode::Char(key_code_char)
                        }
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
