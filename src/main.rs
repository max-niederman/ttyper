mod test;
mod ui;

use test::{Test, results};

use std::io::{self, BufRead};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;
use termion::raw::IntoRawMode;
use termion::event::Key;
use termion::input::TermRead;
use tui::{
    Terminal,
    backend::TermionBackend
};

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
                io::BufReader::new(file).lines().filter_map(|t| t.ok()).collect()
            },
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
        let size = f.size();
        f.render_widget(&test, size);
    })?;

    for k in stdin.keys() {
        let k = k.unwrap();
        match k {
            Key::Ctrl('c') => break,
            Key::Char(_) | Key::Backspace => test.handle_key(k),
            _ => {},
        }

        terminal.draw(|f| {
            let size = f.size();
            f.render_widget(&test, size);
        })?;
    }

    terminal.clear()?;
    println!("{:?}", results::Results::from(&test));
    Ok(())
}
