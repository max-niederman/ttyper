use std::{
    fs::File,
    io::{BufRead, BufReader},
    num::NonZeroUsize,
};

use rand::{
    distributions::{DistIter, Uniform},
    prelude::*,
};
use unicode_segmentation::GraphemeCursor;

use crate::opt::{Command, FileLexer};

/// A trait for types that can be used as test contents.
///
/// The iterator should yield "atoms," the smallest chunks
/// of the test that should not be split across line breaks.
pub trait Contents: Iterator<Item = String> {
    fn restart(&mut self);
}

pub fn generate(env: &crate::Env) -> Box<dyn Contents> {
    match &env.opt.command {
        Command::File { path, lexer } => {
            let raw = std::fs::read_to_string(path).unwrap();
            match lexer {
                FileLexer::ExtendedGraphemeClusters => Box::new(ExtendedGraphemeClusters::new(raw)),
            }
        }
        Command::Words {
            count,
            language: language_name,
            language_cutoff,
        } => {
            let language_name = language_name
                .clone()
                .unwrap_or(env.config.default_language.clone());

            let language: Vec<_> = if language_name.is_file() {
                BufReader::new(File::open(language_name).unwrap())
                    .lines()
                    .take(language_cutoff.unwrap_or(NonZeroUsize::MAX).get())
                    .map(Result::unwrap)
                    .collect()
            } else {
                todo!("builtin languages")
            };

            let contents = Uniform::from(0..language.len())
                .map(move |i| language[i].clone())
                .sample_iter(thread_rng());

            let contents: Box<dyn Contents> = if let Some(count) = count {
                Box::new(Take::new(contents, count.get()))
            } else {
                Box::new(contents)
            };

            contents
        }
    }
}

struct Take<C> {
    contents: C,
    count: usize,
    remaining: usize,
}

impl<C: Contents> Take<C> {
    pub fn new(inner: C, count: usize) -> Self {
        Self {
            count,
            remaining: count,
            contents: inner,
        }
    }
}

impl<C: Iterator> Iterator for Take<C> {
    type Item = C::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            self.contents.next()
        }
    }
}

impl<C: Contents> Contents for Take<C> {
    fn restart(&mut self) {
        self.remaining = self.count;
        self.contents.restart();
    }
}

impl<D, R> Contents for DistIter<D, R, String>
where
    D: Distribution<String>,
    R: Rng,
{
    fn restart(&mut self) {}
}

struct ExtendedGraphemeClusters<S: AsRef<str> = String> {
    string: S,
    cursor: unicode_segmentation::GraphemeCursor,
}

impl<S: AsRef<str>> ExtendedGraphemeClusters<S> {
    pub fn new(string: S) -> Self {
        Self {
            cursor: GraphemeCursor::new(0, string.as_ref().len(), true),
            string,
        }
    }
}

impl<S: AsRef<str>> Iterator for ExtendedGraphemeClusters<S> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.cursor.cur_cursor();
        let end = self
            .cursor
            .next_boundary(self.string.as_ref(), 0)
            .unwrap()?;
        Some(self.string.as_ref()[start..end].to_owned())
    }
}

impl<S: AsRef<str>> Contents for ExtendedGraphemeClusters<S> {
    fn restart(&mut self) {
        self.cursor.set_cursor(0);
    }
}
