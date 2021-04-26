use super::test::{results, Test};

use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use results::Fraction;
use std::iter;
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, BorderType, Borders, Paragraph, Widget},
};

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

trait UsedWidget {}
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

impl<T> DrawInner<T> for SizedBlock<'_>
where
    T: Widget + UsedWidget,
{
    fn draw_inner(&self, content: T, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        content.render(inner, buf);
    }
}

impl Widget for &Test {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Styles
        let title_style = Style::default()
            .patch(Style::default().add_modifier(Modifier::BOLD))
            .patch(Style::default().fg(Color::White));

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(6)])
            .split(area);

        // Sections
        let input = SizedBlock {
            block: Block::default()
                .title(Spans::from(vec![Span::styled("Input", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
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
                        Style::default().fg(match w.progress == w.text {
                            true => Color::Green,
                            false => Color::Red,
                        }),
                    )]
                }))
                .chain(iter::once(vec![
                    Span::styled(
                        self.words[self.current_word]
                            .text
                            .chars()
                            .take(progress_ind)
                            .collect::<String>(),
                        Style::default()
                            .fg(
                                match self.words[self.current_word]
                                    .text
                                    .starts_with(&self.words[self.current_word].progress[..])
                                {
                                    true => Color::Green,
                                    false => Color::Red,
                                },
                            )
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        self.words[self.current_word]
                            .text
                            .chars()
                            .skip(progress_ind)
                            .collect::<String>()
                            + " ",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .chain(self.words[self.current_word + 1..].iter().map(|w| {
                    vec![Span::styled(
                        w.text.clone() + " ",
                        Style::default().fg(Color::Gray),
                    )]
                }));

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
                .title(Spans::from(vec![Span::styled("Prompt", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        );
        target.render(chunks[1], buf);
    }
}

impl Widget for &results::Results {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Styles
        let title_style = Style::default()
            .patch(Style::default().add_modifier(Modifier::BOLD))
            .patch(Style::default().fg(Color::White));

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .split(area);
        let res_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
            .split(chunks[0]);

        let exit = Span::styled(
            "Press any key to finish or 'r' for a another test.",
            Style::default()
                .fg(Color::Gray)
                .add_modifier(Modifier::ITALIC),
        );
        buf.set_span(chunks[1].x, chunks[1].y, &exit, chunks[1].width);

        // Sections
        let mut info_text = Text::styled(
            "",
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Cyan),
        );

        info_text.extend(Text::from(format!(
            "Adjusted WPM: {:.1}",
            self.cps.overall * 12f64 * f64::from(self.accuracy.overall)
        )));
        info_text.extend(Text::from(format!(
            "Accuracy: {:.1}%",
            f64::from(self.accuracy.overall) * 100f64
        )));
        info_text.extend(Text::from(format!(
            "Raw WPM: {:.1}",
            self.cps.overall * 12f64
        )));
        info_text.extend(Text::from(format!(
            "Correct Keypresses: {}",
            self.accuracy.overall
        )));

        // worst keys
        let mut worst_key_display_str = String::from("Worst Keys:");
        let mut worst_keys: Vec<(&KeyEvent, &Fraction)> = self.accuracy.per_key.iter().collect();

        // remove all non chars from the vec
        worst_keys.retain(|(&key, _)| {
            if let KeyCode::Char(_) = key.code {
                return true;
            }
            false
        });

        // Sort by fraction as float
        worst_keys.sort_by(|key_a, key_b| {
            let a_wpm = f64::from(*key_a.1);
            let b_wpm = f64::from(*key_b.1);
            a_wpm.partial_cmp(&b_wpm).unwrap()
        });

        for (key, wpm) in worst_keys.iter().take(std::cmp::min(worst_keys.len(), 5)) {
            if let KeyCode::Char(key_as_char) = key.code {
                worst_key_display_str = format!(
                    "{}\n- {:?} at {:.2}% accuracy",
                    worst_key_display_str,
                    key_as_char,
                    f64::from(**wpm) * 100.0
                );
            }
        }
        info_text.extend(Text::from(worst_key_display_str));

        let info = Paragraph::new(info_text).block(
            Block::default()
                .title(Spans::from(vec![Span::styled("Results", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        info.render(res_chunks[0], buf);
    }
}
