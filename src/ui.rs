use super::test::Test;

use tui::{
    buffer::Buffer,
    layout::{Layout, Constraint, Direction, Rect},
    text::{Spans, Text},
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
            .constraints([Constraint::Min(3), Constraint::Min(6), Constraint::Min(1)])
            .split(area);

        let input = SizedBlock {
            block: Block::default().title("Input").borders(Borders::ALL).border_type(BorderType::Rounded),
            area: chunks[0],
        };
        input.render(buf);
        input.draw_inner(&Spans::from(self.target_progress.clone()), buf);

        let target = SizedBlock {
            block: Block::default().title("Text").borders(Borders::ALL).border_type(BorderType::Rounded),
            area: chunks[1],
        };
        target.render(buf);
        let target_text = Text::from(self.targets[self.current_target].clone());
        target.draw_inner(Paragraph::new(target_text), buf);
    }
}
