use std::{
    num::{self, NonZeroUsize},
    path::PathBuf,
    str::FromStr,
};

#[derive(Debug, clap::Parser)]
pub struct Opt {
    #[command(subcommand)]
    pub command: Command,

    #[clap(long, default_value_os_t = crate::config::default_config_file_path())]
    pub config_file: PathBuf,

    #[clap(long)]
    pub debug: bool,
}

#[derive(Debug, clap::Parser)]
pub enum Command {
    /// Reads test contents from a file.
    File {
        /// Path to the file.
        path: PathBuf,

        /// Lexer with which to read the file.
        #[clap(short, long, default_value = "extended-grapheme-clusters")]
        lexer: FileLexer,
    },
    /// Generates random words for test contents.
    Words {
        /// Number of words to generate.
        count: Option<num::NonZeroUsize>,

        /// Language to sample words from.
        #[clap(short, long)]
        language: Option<PathBuf>,

        /// Take first N words from the language while sampling.
        #[clap(short = 'c', long)]
        language_cutoff: Option<NonZeroUsize>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileLexer {
    ExtendedGraphemeClusters,
}

impl FromStr for FileLexer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "extended-grapheme-clusters" => Ok(Self::ExtendedGraphemeClusters),
            _ => Err(format!("unknown lexer: {}", s)),
        }
    }
}
