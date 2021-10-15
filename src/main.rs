mod test;
mod ui;

use test::{results::Results, Test};

use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, terminal,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rust_embed::RustEmbed;
use std::borrow::Cow;
use std::fs;
use std::io::{self, BufRead};
use std::num;
use std::path::PathBuf;
use std::str;
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
                    .clone()
                    .or_else(|| Some(self.language_dir()?.join(&self.language)))
                    .and_then(|path| fs::read(path).ok())
                    .or_else(|| {
                        Resources::get(&format!("language/{}", self.language)).map(Cow::into_owned)
                    })?;
                let language: Vec<&str> = str::from_utf8(&bytes)
                    .expect("Language file had non-utf8 encoding.")
                    .lines()
                    .collect();

                let mut words: Vec<String> = language
                    .into_iter()
                    .cycle()
                    .take(self.words.into())
                    .map(String::from)
                    .collect();

                let mut rng = thread_rng();
                words.shuffle(&mut rng);

                Some(words)
            }
        }
    }

    /// Installed languages under config directory.
    fn languages(&self) -> Option<Vec<String>> {
        let lang_dir = self.language_dir()?;

        let entries = fs::read_dir(lang_dir)
            .ok()?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .ok()?;

        let mut languages = Vec::new();

        for entry in entries {
            let file = entry.file_name()?;
            let lang = file.to_str()?;

            languages.push(lang.to_string());
        }
        languages.sort();

        Some(languages)
    }

    /// Config directory.
    fn config_dir(&self) -> Option<PathBuf> {
        Some(dirs::config_dir()?.join("ttyper"))
    }

    /// Language directory under condig directory.
    fn language_dir(&self) -> Option<PathBuf> {
        Some(self.config_dir()?.join("language"))
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

    loop {
        if opt.list_languages {
            let langs = opt
                .languages()
                .expect("Couldn't get installed languages under config directory.");

            for lang in langs {
                println!("{}", lang);
            }
            return Ok(());
        }

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
