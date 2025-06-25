use crossterm::event::{KeyCode, KeyModifiers};
use serde::{de, Deserialize};

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct KeyMap {
    #[serde(deserialize_with = "deseralize_key")]
    pub remove_previous_word: Key,
}

fn deseralize_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct KeyVisitor;
    impl<'de> de::Visitor<'de> for KeyVisitor {
        type Value = Key;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            let mut key = Key {
                code: KeyCode::Null,
                modifier: Some(KeyModifiers::NONE),
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
                                        key.modifier = Some(KeyModifiers::CONTROL);
                                    }
                                    'A' => {
                                        key.modifier = Some(KeyModifiers::ALT);
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

            Ok(key)
        }
    }

    return deserializer.deserialize_str(KeyVisitor);
}

#[derive(Debug)]
pub struct Key {
    pub code: KeyCode,
    pub modifier: Option<KeyModifiers>,
}

impl Default for Key {
    fn default() -> Self {
        Self {
            code: KeyCode::Null,
            modifier: Some(KeyModifiers::NONE),
        }
    }
}
