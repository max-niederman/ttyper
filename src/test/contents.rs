use std::str::FromStr;

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

impl Lexer {
    fn consume_lexeme(&self, bytes: impl Iterator<Item = u8>) -> String {
        todo!()
    }
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
