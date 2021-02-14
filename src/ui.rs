use super::test::{
    Test,
    results::{PartialResults}
};

use termion::cursor;
use tui::{
    buffer::Buffer,
    layout::{Layout, Constraint, Direction, Rect},
    text::{Span, Spans, Text},
    style::{Style, Modifier, Color},
    widgets::{Widget, Block, Paragraph, Borders, BorderType},
};

struct SizedBlock<'a> {
    block: Block<'a>,
    area: Rect,
}

impl SizedBlock<'_> {
    fn render(&self, buf: &mut Buffer) {
        // Lifetimes are too difficult for me to understand I guess
        self.block.clone().render(self.area, buf)
    }
}

struct PartialResultsWidget<'a> {
    test: &'a Test,
}

impl Widget for PartialResultsWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let style = Style::default();

        let mut text = Text::default();
        text.extend(Text::styled(format!("Progress: {}", self.test.progress()), style));
        text.extend(Text::styled(format!("WPM: {}", self.test.wpm()), style));
        text.extend(Text::styled(format!("Accuracy: {}%", f32::from(self.test.accuracy()) * 100.0), style));

        Paragraph::new(text).render(area, buf);
    }
}

trait UsedWidget {}
impl UsedWidget for Paragraph<'_> {}
impl UsedWidget for PartialResultsWidget<'_> {}

trait DrawInner<T> {
    fn draw_inner(&self, content: T, buf: &mut Buffer);
}

impl DrawInner<&Spans<'_>> for SizedBlock<'_> {
    fn draw_inner(&self, content: &Spans, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        buf.set_spans(inner.x, inner.y, content, inner.width);
    }
}

impl<T> DrawInner<T> for SizedBlock<'_> where T: Widget + UsedWidget {
    fn draw_inner(&self, content: T, buf: &mut Buffer) {
        let inner = self.block.inner(self.area);
        content.render(inner, buf);
    }
}

impl Widget for &Test {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(6), Constraint::Length(6)])
            .split(area);

        let title_style = Style::default()
            .patch(Style::default().add_modifier(Modifier::BOLD))
            .patch(Style::default().fg(Color::Cyan));

        let input = SizedBlock {
            block: Block::default()
                .title(Spans::from(vec![Span::styled("Input", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
            area: chunks[0],
        };
        input.render(buf);
        print!("{}", cursor::BlinkingBar);
        input.draw_inner(&Spans::from(self.target_progress.clone()), buf);

        let target = SizedBlock {
            block: Block::default()
                .title(Spans::from(vec![Span::styled("Text", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
            area: chunks[1],
        };
        target.render(buf);
        let target_text = Text::from(self.targets[self.current_target].clone());
        target.draw_inner(Paragraph::new(target_text), buf);

        let results = SizedBlock {
            block: Block::default()
                .title(Spans::from(vec![Span::styled("Statistics", title_style)]))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
            area: chunks[2],
        };
        results.render(buf);
        results.draw_inner(PartialResultsWidget { test: self }, buf);
    }
}
