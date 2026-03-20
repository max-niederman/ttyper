use ratatui::{
    style::{Color, Modifier, Style},
    widgets::BorderType,
};
use serde::Deserialize;

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
    pub default: Style,
    pub title: Style,

    // test widget
    pub input_border: Style,
    pub prompt_border: Style,

    #[serde(skip)]
    pub border_type: BorderType,

    pub prompt_correct: Style,
    pub prompt_incorrect: Style,
    pub prompt_untyped: Style,

    pub prompt_current_correct: Style,
    pub prompt_current_incorrect: Style,
    pub prompt_current_untyped: Style,

    pub prompt_cursor: Style,

    // results widget
    pub results_overview: Style,
    pub results_overview_border: Style,

    pub results_worst_keys: Style,
    pub results_worst_keys_border: Style,

    pub results_chart: Style,
    pub results_chart_x: Style,
    pub results_chart_y: Style,

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

            border_type: BorderType::Rounded,

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
