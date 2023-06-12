use std::str::FromStr;
use finl_unicode::grapheme_clusters::Graphemes;

/// A trait for types that can be used as test contents.
///
/// The iterator should yield the smallest chunks of the
/// test that should not be split across line breaks.
pub trait Contents: Iterator<Item = String> + Sized {
    /// Returns the contents of the next, restarted test, if possible.
    fn restart(self) -> Option<Self>;
}

#[derive(Debug, Clone, Copy)]
pub enum Lexer {
    ExtendedGraphemeClusters,
    English,
}

impl Default for Lexer {
    fn default() -> Self {
        Self::ExtendedGraphemeClusters
    }
}

impl FromStr for Lexer {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "english" => Ok(Self::English),
            "extended-grapheme-clusters" => Ok(Self::ExtendedGraphemeClusters),
            _ => Err("invalid lexer language"),
        }
    }
}

struct Utf8Chars<I> {
    bytes: I,
}

impl<I> Iterator for Utf8Chars<I>
where
    I: Iterator<Item = u8>,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        const CONTINUATION_MASK: u8 = 0b0011_1111;

        let first = self.bytes.next()?;
        match first.leading_ones() {
            0 => Some(char::from(first)),
            1 => {
                let first = first & 0b0001_1111;
                let second = self.bytes.next()? & CONTINUATION_MASK;
                char::from_u32((first as u32) << 6 | second as u32)
            }
            2 => {
                let first = first & 0b0000_1111;
                let second = self.bytes.next()? & CONTINUATION_MASK;
                let third = self.bytes.next()? & CONTINUATION_MASK;
                char::from_u32((first as u32) << 12 | (second as u32) << 6 | third as u32)
            }
            3 => {
                let first = first & 0b0000_0111;
                let second = self.bytes.next()? & CONTINUATION_MASK;
                let third = self.bytes.next()? & CONTINUATION_MASK;
                let fourth = self.bytes.next()? & CONTINUATION_MASK;
                char::from_u32(
                    (first as u32) << 18 | (second as u32) << 12 | (third as u32) << 6 | fourth as u32,
                )
            }
            _ => panic!("invalid UTF-8"),
        }
    }
}