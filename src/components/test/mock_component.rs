use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::text::{Line, Span};
use tuirealm::ratatui::widgets::{Block, Borders, Paragraph, Widget};
use tuirealm::{
    command::{Cmd, CmdResult},
    AttrValue, Attribute, Frame, MockComponent, State,
};

use super::spans::words_to_spans;
use super::TestComponent;

impl MockComponent for TestComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let buf = frame.buffer_mut();

        buf.set_style(area, self.theme.default);

        // Chunks
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(6)])
            .split(area);

        // Input Section
        let input_block = Block::default()
            .title(Line::from(vec![Span::styled("Input", self.theme.title)]))
            .borders(Borders::ALL)
            .border_type(self.theme.border_type)
            .border_style(self.theme.input_border);

        let input_inner_area = input_block.inner(chunks[0]);
        input_block.render(chunks[0], buf);

        let input_text = Line::from(self.test.words[self.test.current_word].progress.clone());
        buf.set_line(
            input_inner_area.x,
            input_inner_area.y,
            &input_text,
            input_inner_area.width,
        );

        // Target (Prompt) Section
        let target_lines: Vec<Line> = {
            let words = words_to_spans(&self.test.words, self.test.current_word, &self.theme);

            let mut lines: Vec<Line> = Vec::new();
            let mut current_line: Vec<Span> = Vec::new();
            let mut current_width = 0;
            for word in words {
                let word_width: usize = word.iter().map(|s| s.width()).sum();

                if current_width + word_width > chunks[1].width as usize - 2 {
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
                .title(Span::styled("Prompt", self.theme.title))
                .borders(Borders::ALL)
                .border_type(self.theme.border_type)
                .border_style(self.theme.prompt_border),
        );
        target.render(chunks[1], buf);

        // Cursor positioning
        let inner_x = chunks[0].x + 1;
        let inner_y = chunks[0].y + 1;
        let progress_width =
            Line::from(self.test.words[self.test.current_word].progress.as_str()).width() as u16;
        let max_cursor_x = chunks[0].right().saturating_sub(2);

        frame.set_cursor_position(((inner_x + progress_width).min(max_cursor_x), inner_y));
    }

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }

    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}
