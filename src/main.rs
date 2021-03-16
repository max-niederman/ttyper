mod test;
mod ui;

use test::{results::Results, Test};

use rand::seq::SliceRandom;
use rand::thread_rng;
use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use structopt::StructOpt;
use dirs;
use crossterm::{
    self,
    execute,
    terminal,
    cursor,
    event::{self, Event, KeyCode, KeyModifiers}
};
use tui::{
    backend::CrosstermBackend,
    terminal::Terminal
};

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
                        dirs::config_dir()
                            .expect("Couldn't find configuration directory.")
                            .join("ttyper")
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

fn exit() -> crossterm::Result<()> {
    execute!(
        io::stdout(),
        cursor::RestorePosition,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )?;

    Ok(())
}

fn main() -> crossterm::Result<()> {
    let opt = Opt::from_args();

    let mut test = Test::new(opt.gen_contents());

    let mut stdout = io::stdout();
    execute!(
        stdout, 
        cursor::Hide,
        cursor::SavePosition, 
        terminal::EnterAlternateScreen,
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&test, f.size());
    })?;

    // Enable raw mode to read keys
    terminal::enable_raw_mode()?;

    loop {
        match event::read()? {
            Event::Key(key) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match key.code {
                        KeyCode::Char('c') => return exit(),
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Esc => break,
                    _ => test.handle_key(key)
                }

                if test.complete {
                    break;
                }

                terminal.draw(|f| {
                    f.render_widget(&test, f.size());
                })?;
            },
            Event::Resize(_, _) => {
                terminal.draw(|f| {
                    f.render_widget(&test, f.size());
                })?;
            },
            _ => {}
        }
            
    }

    // Draw results
    let results = Results::from(&test);
    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&results, f.size());
    })?;

    // Wait for keypress
    loop {
        match event::read()? {
            Event::Key(_) => break,
            _ => {}
        }
    }
    
    terminal::disable_raw_mode()?;

    exit()
} 
