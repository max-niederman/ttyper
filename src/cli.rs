use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;
use std::{num, str};

#[derive(Debug, Parser)]
#[command(about, version)]
pub struct Opt {
    /// Read test contents from the specified file, or "-" for stdin
    #[arg(value_name = "PATH")]
    pub contents: Option<PathBuf>,

    #[arg(short, long)]
    pub debug: bool,

    /// Specify word count
    #[arg(short, long, value_name = "N", default_value = "50")]
    pub words: num::NonZeroUsize,

    /// Use config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Specify test language in file
    #[arg(long, value_name = "PATH")]
    pub language_file: Option<PathBuf>,

    /// Specify test language
    #[arg(short, long, value_name = "LANG")]
    pub language: Option<String>,

    /// List installed languages
    #[arg(long)]
    pub list_languages: bool,

    /// Disable backtracking to completed words
    #[arg(long)]
    pub no_backtrack: bool,

    /// Enable sudden death mode to restart on first error
    #[arg(long)]
    pub sudden_death: bool,

    /// Disable backspace
    #[arg(long)]
    pub no_backspace: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
