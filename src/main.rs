mod test;

use std::{num::{self, NonZeroUsize}, path::PathBuf};

#[derive(Debug, clap::Parser)]
struct Opt {
    #[command(subcommand)]
    command: Command,

    #[clap(long)]
    debug: bool,
}

#[derive(Debug, clap::Parser)]
enum Command {
    /// Reads test contents from a file.
    File {
        path: PathBuf,
    },
    /// Generates random words for test contents.
    Words {
        /// Number of words to generate.
        count: num::NonZeroUsize,

        /// Language to sample words from.
        #[clap(short, long)]
        language: Option<String>,

        /// Take first N words from the language while sampling.
        #[clap(short = 'c', long)]
        language_cutoff: Option<NonZeroUsize>,
    }
}

fn main() {
    let opt = <Opt as clap::Parser>::parse();
    if opt.debug {
        dbg!(opt);
    }
}