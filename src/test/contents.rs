use std::str::FromStr;

/// A trait for types that can be used as test contents.
///
/// The iterator should yield the smallest chunks of the
/// test that should not be split across line breaks.
pub trait Contents<I = String>: Iterator<Item = I> + Sized {
    /// Returns the next, restarted test, if possible.
    fn restart(self) -> Option<Self>;
}

pub struct Lexed<C: Contents<char>> {
    inner: C,
    language: LexerLanguage,
}

impl<C> Iterator for Lexed<C>
where
    C: Contents<char>
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<C> Contents<String> for Lexed<C>
where
    C: Contents<char>
{
    fn restart(self) -> Option<Self> {
        Some(Self {
            inner: self.inner.restart()?,
            language: self.language,
        })
    }
}

pub enum LexerLanguage {
    English,
    ExtendedGraphemeClusters
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
