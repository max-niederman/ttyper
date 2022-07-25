use crate::config::Theme;

use super::test::{results, Test};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use results::Fraction;
use std::iter;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    text::{Span, Spans, Text},
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};

// Convert CPS to WPM (clicks per second)
const WPM_PER_CPS: f64 = 12.0;

// Width of the moving average window for the WPM chart
const WPM_SMA_WIDTH: usize = 10;

#[derive(Clone)]
struct SizedBlock<'a> {
    block: Block<'a>,
    area: Rect,
}

impl SizedBlock<'_> {
    fn render(self, buf: &mut Buffer) {
        self.block.render(self.area, buf)
    }
}

trait UsedWidget: Widget {}
impl UsedWidget for Paragraph<'_> {}

trait DrawInner<T> {
    fn draw_inner(&self, content: T, buf: &mut Buffer);
}

impl DrawInner<&Spans<'_>> for SizedBlock<'_> {
    fn draw_inner(&self, content: &Spans, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        buf.set_spans(inner.x, inner.y, content, inner.width);
    }
}

impl<T: UsedWidget> DrawInner<T> for SizedBlock<'_> {
    fn draw_inner(&self, content: T, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        content.render(inner, buf);
    }
}

pub trait ThemedWidget {
    fn render(self, area: Rect, buf: &mut Buffer, theme: &Theme);
}

pub struct Themed<'t, W: ?Sized> {
    theme: &'t Theme,
    widget: W,
}
impl<'t, W: ThemedWidget> Widget for Themed<'t, W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.widget.render(area, buf, self.theme)
    }
}
impl Theme {
    pub fn apply_to<W>(&self, widget: W) -> Themed<'_, W> {
        Themed {
            theme: self,
            widget,
        }
    }
}

impl ThemedWidget for &Test {
    fn render(self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        buf.set_style(area, theme.default);

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(6)])
            .split(area);

        // Sections
        let input = SizedBlock {
            block: Block::default()
                .title(Spans::from(vec![Span::styled("Input", theme.title)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.input_border),
            area: chunks[0],
        };
        input.draw_inner(
            &Spans::from(self.words[self.current_word].progress.clone()),
            buf,
        );
        input.render(buf);

        let target_lines: Vec<Spans> = {
            let progress_ind = self.words[self.current_word]
                .progress
                .len()
                .min(self.words[self.current_word].text.len());
            let words = iter::empty::<Vec<Span>>()
                .chain(self.words[..self.current_word].iter().map(|w| {
                    vec![Span::styled(
                        w.text.clone() + " ",
                        if w.progress == w.text {
                            theme.prompt_correct
                        } else {
                            theme.prompt_incorrect
                        },
                    )]
                }))
                .chain(iter::once(vec![
                    Span::styled(
                        self.words[self.current_word]
                            .text
                            .chars()
                            .take(progress_ind)
                            .collect::<String>(),
                        if self.words[self.current_word]
                            .text
                            .starts_with(&self.words[self.current_word].progress[..])
                        {
                            theme.prompt_current_correct
                        } else {
                            theme.prompt_current_incorrect
                        },
                    ),
                    Span::styled(
                        self.words[self.current_word]
                            .text
                            .chars()
                            .skip(progress_ind)
                            .collect::<String>()
                            + " ",
                        theme.prompt_current_untyped,
                    ),
                ]))
                .chain(
                    self.words[self.current_word + 1..]
                        .iter()
                        .map(|w| vec![Span::styled(w.text.clone() + " ", theme.prompt_untyped)]),
                );

            let mut lines: Vec<Spans> = Vec::new();
            let mut current_line: Vec<Span> = Vec::new();
            let mut current_width = 0;
            for word in words {
                let word_width: usize = word.iter().map(|s| s.width()).sum();

                if current_width + word_width > chunks[1].width as usize - 2 {
                    current_line.push(Span::raw("\n"));
                    lines.push(Spans::from(current_line.clone()));
                    current_line.clear();
                    current_width = 0;
                }

                current_line.extend(word);
                current_width += word_width;
            }
            lines.push(Spans::from(current_line));

            lines
        };
        let target = Paragraph::new(target_lines).block(
            Block::default()
                .title(Span::styled("Prompt", theme.title))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.prompt_border),
        );
        target.render(chunks[1], buf);
    }
}

impl ThemedWidget for &results::Results {
    fn render(self, area: Rect, buf: &mut Buffer, theme: &Theme) {
        buf.set_style(area, theme.default);

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        let res_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1) // Graph looks tremendously better with just a little margin
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(chunks[0]);
        let info_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
            .split(res_chunks[0]);

        let exit = Span::styled(
            "Press 'q' to quit or 'r' for another test.",
            theme.results_restart_prompt,
        );
        buf.set_span(chunks[1].x, chunks[1].y, &exit, chunks[1].width);

        // Sections
        let mut overview_text = Text::styled("", theme.results_overview);
        overview_text.extend([
            Spans::from(format!(
                "Adjusted WPM: {:.1}",
                self.timing.overall_cps * WPM_PER_CPS * f64::from(self.accuracy.overall)
            )),
            Spans::from(format!(
                "Accuracy: {:.1}%",
                f64::from(self.accuracy.overall) * 100f64
            )),
            Spans::from(format!(
                "Raw WPM: {:.1}",
                self.timing.overall_cps * WPM_PER_CPS
            )),
            Spans::from(format!("Correct Keypresses: {}", self.accuracy.overall)),
        ]);
        let overview = Paragraph::new(overview_text).block(
            Block::default()
                .title(Span::styled("Overview", theme.title))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.results_overview_border),
        );
        overview.render(info_chunks[0], buf);

        let mut worst_keys: Vec<(&KeyEvent, &Fraction)> = self
            .accuracy
            .per_key
            .iter()
            .filter(|(key, _)| matches!(key.code, KeyCode::Char(_)))
            .collect();
        worst_keys.sort_unstable_by_key(|x| x.1);

        let mut worst_text = Text::styled("", theme.results_worst_keys);
        worst_text.extend(
            worst_keys
                .iter()
                .take(5)
                .filter_map(|(key, acc)| {
                    if let KeyCode::Char(character) = key.code {
                        Some(format!(
                            "- {} at {:.1}% accuracy",
                            character,
                            f64::from(**acc) * 100.0,
                        ))
                    } else {
                        None
                    }
                })
                .map(Spans::from),
        );
        let worst = Paragraph::new(worst_text).block(
            Block::default()
                .title(Span::styled("Worst Keys", theme.title))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.results_worst_keys_border),
        );
        worst.render(info_chunks[1], buf);

        let wpm_sma: Vec<(f64, f64)> = self
            .timing
            .per_event
            .windows(WPM_SMA_WIDTH)
            .enumerate()
            .map(|(i, window)| {
                (
                    (i + WPM_SMA_WIDTH) as f64,
                    window.len() as f64 / window.iter().copied().sum::<f64>() * WPM_PER_CPS,
                )
            })
            .collect();

        let wpm_sma_min = wpm_sma
            .iter()
            .map(|(_, x)| x)
            .fold(f64::INFINITY, |a, &b| a.min(b));
        let wpm_sma_max = wpm_sma
            .iter()
            .map(|(_, x)| x)
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        let wpm_datasets = vec![Dataset::default()
            .name("WPM")
            .marker(Marker::Braille)
            .graph_type(GraphType::Line)
            .style(theme.results_chart)
            .data(&wpm_sma)];

        let wpm_chart = Chart::new(wpm_datasets)
            .block(Block::default().title(vec![Span::styled("Chart", theme.title)]))
            .x_axis(
                Axis::default()
                    .title(Span::styled("Keypresses", theme.results_chart_x))
                    .bounds([0.0, self.timing.per_event.len() as f64]),
            )
            .y_axis(
                Axis::default()
                    .title(Span::styled(
                        "WPM (10-keypress rolling average)",
                        theme.results_chart_y,
                    ))
                    .bounds([wpm_sma_min, wpm_sma_max])
                    .labels(
                        (wpm_sma_min as u16..wpm_sma_max as u16)
                            .step_by(5)
                            .map(|n| Span::raw(format!("{}", n)))
                            .collect(),
                    ),
            );
        wpm_chart.render(res_chunks[1], buf);
    }
}
