mod config;
mod test;
mod ui;

use test::{results::Results, Test};

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use structopt::StructOpt;
use termion::{
    input::TermRead,
    screen::AlternateScreen,
    raw::IntoRawMode,
    event::Key
};
use tui::{backend::TermionBackend, terminal::Terminal};

#[derive(Debug, StructOpt)]
#[structopt(name = "ttyper", about = "Terminal-based typing test.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    contents: Option<PathBuf>,

    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "50")]
    words: usize,

    #[structopt(long, parse(from_os_str))]
    language_file: Option<PathBuf>,

    #[structopt(short, long, default_value = "english200")]
    language: String,
}

impl Opt {
    fn gen_contents(&self) -> Vec<String> {
        match &self.contents {
            Some(path) => {
                let file = fs::File::open(path).expect("Error reading language file.");
                let lines: Vec<String> = io::BufReader::new(file)
                    .lines()
                    .filter_map(Result::ok)
                    .collect();
                lines
                    .iter()
                    .flat_map(|line| line.split_whitespace())
                    .map(String::from)
                    .collect()
            },
            None => {
                let language: Vec<String> = {
                    let path = self.language_file.clone().unwrap_or_else(|| {
                        config::get_path()
                            .join("language")
                            .join(&self.language)
                    });
                    let file = fs::File::open(path).expect("Error reading language file.");
                    io::BufReader::new(file)
                        .lines()
                        .filter_map(Result::ok)
                        .collect()
                };

                let mut words: Vec<String> = language.into_iter().cycle().take(self.words).collect();

                let mut rng = thread_rng();
                words.shuffle(&mut rng);

                words
            }
        }
    }
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    let mut test = Test::new(opt.gen_contents());

    let stdin = io::stdin();
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(AlternateScreen::from(stdout));
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
            _ => test.handle_key(key),
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
