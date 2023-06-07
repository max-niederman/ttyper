use ratatui::style::{Color, Modifier, Style};
use serde::{
    de::{self, IntoDeserializer},
    Deserialize,
};

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    pub default_language: String,
    pub theme: Theme,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_language: "english200".into(),
            theme: Theme::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Theme {
    #[serde(deserialize_with = "deserialize_style")]
    pub default: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub title: Style,

    // test widget
    #[serde(deserialize_with = "deserialize_style")]
    pub input_border: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_border: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_correct: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_incorrect: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_untyped: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_current_correct: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_current_incorrect: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_current_untyped: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub prompt_cursor: Style,

    // results widget
    #[serde(deserialize_with = "deserialize_style")]
    pub results_overview: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub results_overview_border: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub results_worst_keys: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub results_worst_keys_border: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub results_chart: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub results_chart_x: Style,
    #[serde(deserialize_with = "deserialize_style")]
    pub results_chart_y: Style,

    #[serde(deserialize_with = "deserialize_style")]
    pub results_restart_prompt: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            default: Style::default(),

            title: Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),

            input_border: Style::default().fg(Color::Cyan),
            prompt_border: Style::default().fg(Color::Green),

            prompt_correct: Style::default().fg(Color::Green),
            prompt_incorrect: Style::default().fg(Color::Red),
            prompt_untyped: Style::default().fg(Color::Gray),

            prompt_current_correct: Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
            prompt_current_incorrect: Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            prompt_current_untyped: Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),

            prompt_cursor: Style::default().add_modifier(Modifier::UNDERLINED),

            results_overview: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            results_overview_border: Style::default().fg(Color::Cyan),

            results_worst_keys: Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
            results_worst_keys_border: Style::default().fg(Color::Cyan),

            results_chart: Style::default().fg(Color::Cyan),
            results_chart_x: Style::default().fg(Color::Cyan),
            results_chart_y: Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::BOLD),

            results_restart_prompt: Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        }
    }
}

fn deserialize_style<'de, D>(deserializer: D) -> Result<Style, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StyleVisitor;
    impl<'de> de::Visitor<'de> for StyleVisitor {
        type Value = Style;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string describing a text style")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            let (colors, modifiers) = value.split_once(';').unwrap_or((value, ""));
            let (fg, bg) = colors.split_once(':').unwrap_or((colors, "none"));

            let mut style = Style {
                fg: match fg {
                    "none" | "" => None,
                    _ => Some(deserialize_color(fg.into_deserializer())?),
                },
                bg: match bg {
                    "none" | "" => None,
                    _ => Some(deserialize_color(bg.into_deserializer())?),
                },
                ..Default::default()
            };

            for modifier in modifiers.split_terminator(';') {
                style = style.add_modifier(match modifier {
                    "bold" => Modifier::BOLD,
                    "crossed_out" => Modifier::CROSSED_OUT,
                    "dim" => Modifier::DIM,
                    "hidden" => Modifier::HIDDEN,
                    "italic" => Modifier::ITALIC,
                    "rapid_blink" => Modifier::RAPID_BLINK,
                    "slow_blink" => Modifier::SLOW_BLINK,
                    "reversed" => Modifier::REVERSED,
                    "underlined" => Modifier::UNDERLINED,
                    _ => {
                        return Err(E::invalid_value(
                            de::Unexpected::Str(modifier),
                            &"a style modifier",
                        ))
                    }
                });
            }

            Ok(style)
        }
    }

    deserializer.deserialize_str(StyleVisitor)
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct ColorVisitor;
    impl<'de> de::Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a color name or hexadecimal color code")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
            match value {
                "reset" => Ok(Color::Reset),
                "black" => Ok(Color::Black),
                "white" => Ok(Color::White),
                "red" => Ok(Color::Red),
                "green" => Ok(Color::Green),
                "yellow" => Ok(Color::Yellow),
                "blue" => Ok(Color::Blue),
                "magenta" => Ok(Color::Magenta),
                "cyan" => Ok(Color::Cyan),
                "gray" => Ok(Color::Gray),
                "darkgray" => Ok(Color::DarkGray),
                "lightred" => Ok(Color::LightRed),
                "lightgreen" => Ok(Color::LightGreen),
                "lightyellow" => Ok(Color::LightYellow),
                "lightblue" => Ok(Color::LightBlue),
                "lightmagenta" => Ok(Color::LightMagenta),
                "lightcyan" => Ok(Color::LightCyan),
                _ => {
                    if value.len() == 6 {
                        let parse_error = |_| E::custom("color code was not valid hexadecimal");

                        Ok(Color::Rgb(
                            u8::from_str_radix(&value[0..2], 16).map_err(parse_error)?,
                            u8::from_str_radix(&value[2..4], 16).map_err(parse_error)?,
                            u8::from_str_radix(&value[4..6], 16).map_err(parse_error)?,
                        ))
                    } else {
                        Err(E::invalid_value(
                            de::Unexpected::Str(value),
                            &"a color name or hexadecimal color code",
                        ))
                    }
                }
            }
        }
    }

    deserializer.deserialize_str(ColorVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_basic_colors() {
        fn color(string: &str) -> Color {
            deserialize_color(de::IntoDeserializer::<de::value::Error>::into_deserializer(
                string,
            ))
            .expect("failed to deserialize color")
        }

        assert_eq!(color("black"), Color::Black);
        assert_eq!(color("000000"), Color::Rgb(0, 0, 0));
        assert_eq!(color("ffffff"), Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(color("FFFFFF"), Color::Rgb(0xff, 0xff, 0xff));
    }

    #[test]
    fn deserializes_styles() {
        fn style(string: &str) -> Style {
            deserialize_style(de::IntoDeserializer::<de::value::Error>::into_deserializer(
                string,
            ))
            .expect("failed to deserialize style")
        }

        assert_eq!(style("none"), Style::default());
        assert_eq!(style("none:none"), Style::default());
        assert_eq!(style("none:none;"), Style::default());

        assert_eq!(style("black"), Style::default().fg(Color::Black));
        assert_eq!(
            style("black:white"),
            Style::default().fg(Color::Black).bg(Color::White)
        );

        assert_eq!(
            style("none;bold"),
            Style::default().add_modifier(Modifier::BOLD)
        );
        assert_eq!(
            style("none;bold;italic;underlined;"),
            Style::default()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC)
                .add_modifier(Modifier::UNDERLINED)
        );

        assert_eq!(
            style("00ff00:000000;bold;dim;italic;slow_blink"),
            Style::default()
                .fg(Color::Rgb(0, 0xff, 0))
                .bg(Color::Rgb(0, 0, 0))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::DIM)
                .add_modifier(Modifier::ITALIC)
                .add_modifier(Modifier::SLOW_BLINK)
        );
    }
}
