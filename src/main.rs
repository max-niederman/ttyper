mod language;
mod test;
mod ui;

use test::{results::Results, Test};

use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use rand::thread_rng;
use rand::seq::SliceRandom;
use structopt::StructOpt;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::{backend::TermionBackend, terminal::Terminal};

#[derive(Debug, StructOpt)]
#[structopt(name = "ttyper", about = "Terminal-based typing test.")]
struct Opt {
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "50")]
    words: usize,

    #[structopt(parse(from_os_str))]
    language_file: Option<PathBuf>,

    #[structopt(short, long, default_value = "english200")]
    language: String,
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    let mut test = {
        let words = match opt.language_file {
            Some(path) => {
                let file = fs::File::open(path).expect("Error reading test input file.");
                io::BufReader::new(file)
                    .lines()
                    .filter_map(|t| t.ok())
                    .collect()
            }
            None => language::get_words(opt.language)?,
        };

        let mut rng = thread_rng();
        let shuffled = words.choose_multiple(&mut rng, opt.words).collect();
        println!("{:?}", shuffled);

        Test::new(shuffled)
    };

    let stdin = io::stdin();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&test, f.size());
    })?;

    for key in stdin.keys() {
        let key = key.unwrap();
        match key {
            Key::Esc => break,
            Key::Ctrl('c') => return Ok(()),
            _ => test.handle_key(key)
        }

        if test.complete {
            break;
        }

        terminal.draw(|f| {
            f.render_widget(&test, f.size());
        })?;
    }

    // Draw results
    let results = Results::from(&test);
    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&results, f.size());
    })?;

    // Wait for keypress
    io::stdin().keys().next();

    Ok(())
}
