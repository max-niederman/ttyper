use std::str::FromStr;

/// A trait for types that can be used as test contents.
///
/// The iterator should yield the smallest chunks of the
/// test that should not be split across line breaks.
pub trait Contents: Iterator<Item = String> + MaybeRestartable {}

pub trait MaybeRestartable: Sized {
    /// Returns a restarted value, if possible.
    fn restart(self) -> Option<Self>;
}

#[derive(Debug, Clone, Copy)]
pub struct Lexed<I> {
    inner: I,
    language: LexerLanguage,
}

impl<I> Iterator for Lexed<I>
where
    I: Iterator<Item = char>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<I> MaybeRestartable for Lexed<I>
where
    I: MaybeRestartable,
{
    fn restart(self) -> Option<Self> {
        Some(Self {
            inner: self.inner.restart()?,
            language: self.language,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LexerLanguage {
    English,
    ExtendedGraphemeClusters,
}

impl Default for LexerLanguage {
    fn default() -> Self {
        Self::ExtendedGraphemeClusters
    }
}

impl FromStr for LexerLanguage {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "english" => Ok(Self::English),
            "extended-grapheme-clusters" => Ok(Self::ExtendedGraphemeClusters),
            _ => Err("invalid lexer language"),
        }
    }
}
