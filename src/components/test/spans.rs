use super::status::{split_current_word, split_typed_word, word_parts_to_spans, Status};
use super::word::TestWord;
use crate::config::Theme;
use tuirealm::ratatui::text::Span;

pub fn words_to_spans<'a>(
    words: &'a [TestWord],
    current_word: usize,
    theme: &'a Theme,
) -> Vec<Vec<Span<'a>>> {
    let mut spans = Vec::new();

    for word in &words[..current_word] {
        let parts = split_typed_word(word);
        spans.push(word_parts_to_spans(parts, theme));
    }

    if current_word < words.len() {
        let parts_current = split_current_word(&words[current_word]);
        spans.push(word_parts_to_spans(parts_current, theme));

        for word in &words[current_word + 1..] {
            let parts = vec![(word.text.clone(), Status::Untyped)];
            spans.push(word_parts_to_spans(parts, theme));
        }
    }
    spans
}
