use crate::config::Theme;

use super::test::{results, Test};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    symbols::Marker,
    text::{Line, Span, Text},
    widgets::{Axis, Block, BorderType, Borders, Chart, Dataset, GraphType, Paragraph, Widget},
};
use results::Fraction;
use std::iter;

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

impl DrawInner<&Line<'_>> for SizedBlock<'_> {
    fn draw_inner(&self, content: &Line, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        buf.set_line(inner.x, inner.y, content, inner.width);
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
                .title(Line::from(vec![Span::styled("Input", theme.title)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.input_border),
            area: chunks[0],
        };
        input.draw_inner(
            &Line::from(self.words[self.current_word].progress.clone()),
            buf,
        );
        input.render(buf);

        let target_lines: Vec<Line> = {
            let words = iter::empty::<Vec<Span>>()
                // already typed words
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
                // current word
                .chain({
                    let text = &self.words[self.current_word].text;
                    let prog = &self.words[self.current_word].progress;
                    let text_count = text.chars().count();
                    let prog_count = prog.chars().count();

                    // Find the index for the first incorrect character
                    let mut incorrect_ind = 0;
                    for (idx, chr) in prog.char_indices() {
                        if !text.get(idx..).unwrap().starts_with(chr) {
                            break;
                        }
                        incorrect_ind = idx + chr.len_utf8();
                    }

                    let (cursor_ind, chr) = text.char_indices()
                      .nth(prog_count.min(text_count))
                      .unwrap_or((text.len(), '\0'));
                    let untyped_ind = cursor_ind + chr.len_utf8();

                    iter::once(vec![
                        Span::styled(
                            text.get(..incorrect_ind).unwrap(),
                            theme.prompt_current_correct,
                        ),
                        Span::styled(
                            text.get(incorrect_ind..cursor_ind).unwrap(),
                            theme.prompt_current_incorrect,
                        ),
                        Span::styled(
                            text.get(cursor_ind..untyped_ind).unwrap_or(""),
                            theme.prompt_current_untyped.patch(theme.prompt_cursor),
                        ),
                        Span::styled(
                            text.get(untyped_ind..).unwrap_or(""),
                            theme.prompt_current_untyped,
                        ),
                        Span::styled(
                            " ", // Delimiting space
                            if prog_count > text_count {
                                // There are redundant letters at the end
                                theme.prompt_current_incorrect.patch(theme.prompt_cursor)
                            } else if prog_count == text_count {
                                theme.prompt_current_untyped.patch(theme.prompt_cursor)
                            } else {
                                theme.prompt_current_untyped
                            },
                        ),
                    ])
                })
                // remaining words
                .chain(
                    self.words[self.current_word + 1..]
                        .iter()
                        .map(|w| vec![Span::styled(w.text.clone() + " ", theme.prompt_untyped)]),
                );

            let mut lines: Vec<Line> = Vec::new();
            let mut current_line: Vec<Span> = Vec::new();
            let mut current_width = 0;
            for word in words {
                let word_width: usize = word.iter().map(|s| s.width()).sum();

                if current_width + word_width > chunks[1].width as usize - 2 {
                    current_line.push(Span::raw("\n"));
                    lines.push(Line::from(current_line.clone()));
                    current_line.clear();
                    current_width = 0;
                }

                current_line.extend(word);
                current_width += word_width;
            }
            lines.push(Line::from(current_line));

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
            Line::from(format!(
                "Adjusted WPM: {:.1}",
                self.timing.overall_cps * WPM_PER_CPS * f64::from(self.accuracy.overall)
            )),
            Line::from(format!(
                "Accuracy: {:.1}%",
                f64::from(self.accuracy.overall) * 100f64
            )),
            Line::from(format!(
                "Raw WPM: {:.1}",
                self.timing.overall_cps * WPM_PER_CPS
            )),
            Line::from(format!("Correct Keypresses: {}", self.accuracy.overall)),
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
                .map(Line::from),
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

// FIXME: replace with `str::ceil_char_boundary` when stable
fn ceil_char_boundary(string: &str, index: usize) -> usize {
    if string.is_char_boundary(index) {
        index
    } else {
        ceil_char_boundary(string, index + 1)
    }
}
