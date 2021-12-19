mod test;
mod ui;

use test::{results::Results, Test};

use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use rand::{seq::SliceRandom, thread_rng};
use rust_embed::RustEmbed;
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead},
    num,
    path::PathBuf,
    str,
};
use structopt::StructOpt;
use tui::{backend::CrosstermBackend, terminal::Terminal};

#[derive(RustEmbed)]
#[folder = "resources/runtime"]
struct Resources;

#[derive(Debug, StructOpt)]
#[structopt(name = "ttyper", about = "Terminal-based typing test.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    contents: Option<PathBuf>,

    #[allow(dead_code)]
    #[structopt(short, long)]
    debug: bool,

    #[structopt(short, long, default_value = "50")]
    words: num::NonZeroUsize,

    #[structopt(long, parse(from_os_str))]
    language_file: Option<PathBuf>,

    #[structopt(short, long, default_value = "english200")]
    language: String,

    /// List installed languages
    #[structopt(long)]
    list_languages: bool,
}

impl Opt {
    fn gen_contents(&self) -> Option<Vec<String>> {
        match &self.contents {
            Some(path) => {
                let file = fs::File::open(path).expect("Error reading language file.");
                let lines: Vec<String> = io::BufReader::new(file)
                    .lines()
                    .filter_map(Result::ok)
                    .collect();
                Some(lines.iter().map(String::from).collect())
            }
            None => {
                let bytes: Vec<u8> = self
                    .language_file
                    .as_ref()
                    .map(|p| fs::read(p).ok())
                    .flatten()
                    .or_else(|| fs::read(self.language_dir().join(&self.language)).ok())
                    .or_else(|| {
                        Resources::get(&format!("language/{}", self.language))
                            .map(|f| f.data.into_owned())
                    })?;

                let mut language: Vec<&str> = str::from_utf8(&bytes)
                    .expect("Language file had non-utf8 encoding.")
                    .lines()
                    .collect();

                let mut rng = thread_rng();
                language.shuffle(&mut rng);

                Some(
                    language
                        .into_iter()
                        .cycle()
                        .take(self.words.into())
                        .map(String::from)
                        .collect(),
                )
            }
        }
    }

    /// Installed languages under config directory
    fn languages(&self) -> io::Result<Vec<OsString>> {
        Ok(self
            .language_dir()
            .read_dir()?
            .filter_map(Result::ok)
            .map(|e| e.file_name())
            .collect())
    }

    /// Config directory
    fn config_dir(&self) -> PathBuf {
        dirs::config_dir()
            .expect("Failed to find config directory.")
            .join("ttyper")
    }

    /// Language directory under config directory
    fn language_dir(&self) -> PathBuf {
        self.config_dir().join("language")
    }
}

fn run_test(mut test: Test) -> crossterm::Result<bool> {
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
                    if let KeyCode::Char('c') = key.code {
                        return Ok(false);
                    };
                }

                match key.code {
                    KeyCode::Esc => break,
                    _ => test.handle_key(key),
                }

                if test.complete {
                    break;
                }

                terminal.draw(|f| {
                    f.render_widget(&test, f.size());
                })?;
            }
            Event::Resize(_, _) => {
                terminal.draw(|f| {
                    f.render_widget(&test, f.size());
                })?;
            }
            _ => {}
        }
    }

    // Draw results
    let results = Results::from(test);
    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&results, f.size());
    })?;
    Ok(true)
}

fn exit() -> crossterm::Result<()> {
    terminal::disable_raw_mode()?;
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

    if opt.list_languages {
        opt.languages()
            .expect("Couldn't get installed languages under config directory.")
            .iter()
            .for_each(|name| println!("{}", name.to_str().expect("Ill-formatted language name.")));
        return Ok(());
    }

    loop {
        let contents = opt.gen_contents().expect(
            "Couldn't get test contents. Make sure the specified language actually exists.",
        );
        if contents.is_empty() {
            println!("Test contents were empty. Exiting...");
            return Ok(());
        };

        if run_test(Test::new(contents))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    modifiers: KeyModifiers::NONE,
                }) => (),
                _ => break,
            }
        } else {
            return exit();
        }
    }
    exit()
}
