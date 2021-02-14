mod test;
mod ui;

use test::{results::Results, Test};

use std::fs;
use std::io::{self, Read, BufRead};
use std::path::PathBuf;
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

    // TODO: Add option to download text automatically
    #[structopt(parse(from_os_str))]
    test_contents: Option<PathBuf>,
}

fn main() -> Result<(), io::Error> {
    let opt = Opt::from_args();

    let mut test = {
        let words = match opt.test_contents {
            Some(path) => {
                let file = fs::File::open(path).expect("Error reading test input file.");
                io::BufReader::new(file)
                    .lines()
                    .filter_map(|t| t.ok())
                    .collect()
            }
            None => unimplemented!(),
        };

        Test::new(words)
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
            Key::Ctrl('c') => break,
            Key::Char(_) | Key::Backspace => test.handle_key(key),
            _ => {}
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
    {
        let mut buf = [0; 1];
        io::stdin().read(&mut buf)?;
    }

    Ok(())
}
