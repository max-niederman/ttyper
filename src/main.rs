mod config;
mod test;
mod ui;

use config::Config;
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

    #[structopt(short, long)]
    debug: bool,

    /// Specify word count
    #[structopt(short, long, default_value = "50")]
    words: num::NonZeroUsize,

    /// Use config file
    #[structopt(short, long)]
    config: Option<PathBuf>,

    /// Specify test language in file
    #[structopt(long, parse(from_os_str))]
    language_file: Option<PathBuf>,

    /// Specify test language
    #[structopt(short, long)]
    language: Option<String>,

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
                let lang_name = self
                    .language
                    .clone()
                    .unwrap_or_else(|| self.config().default_language);

                let bytes: Vec<u8> = self
                    .language_file
                    .as_ref()
                    .map(fs::read)
                    .map(Result::ok)
                    .flatten()
                    .or_else(|| fs::read(self.language_dir().join(&lang_name)).ok())
                    .or_else(|| {
                        Resources::get(&format!("language/{}", &lang_name))
                            .map(|f| f.data.into_owned())
                    })?;

                let language: Vec<&str> = str::from_utf8(&bytes)
                    .expect("Language file had non-utf8 encoding.")
                    .lines()
                    .collect();

                let mut contents: Vec<_> = language
                    .into_iter()
                    .cycle()
                    .take(self.words.get())
                    .map(ToOwned::to_owned)
                    .collect();

                contents.shuffle(&mut thread_rng());

                Some(contents)
            }
        }
    }

    /// Configuration
    fn config(&self) -> Config {
        fs::read(
            self.config
                .clone()
                .unwrap_or_else(|| self.config_dir().join("config.toml")),
        )
        .map(|bytes| toml::from_slice(&bytes).expect("Configuration was ill-formed."))
        .unwrap_or_default()
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

fn run_test(config: &Config, mut test: Test) -> crossterm::Result<bool> {
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
        f.render_widget(config.theme.apply_to(&test), f.size());
    })?;

    // Enable raw mode to read keys
    terminal::enable_raw_mode()?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
            }) => return Ok(false),

            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
            }) => break,

            Event::Key(key) => {
                test.handle_key(key);

                if test.complete {
                    break;
                }

                terminal.draw(|f| {
                    f.render_widget(config.theme.apply_to(&test), f.size());
                })?;
            }

            Event::Resize(_, _) => {
                terminal.draw(|f| {
                    f.render_widget(config.theme.apply_to(&test), f.size());
                })?;
            }
            _ => {}
        }
    }

    // Draw results
    let results = Results::from(test);
    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(config.theme.apply_to(&results), f.size());
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
    let config = opt.config();

    if opt.debug {
        dbg!(&config);
    }

    if opt.list_languages {
        opt.languages()
            .expect("Couldn't get installed languages under config directory. Make sure the config directory exists.")
            .iter()
            .for_each(|name| println!("{}", name.to_str().expect("Ill-formatted language name.")));
        return Ok(());
    }

    'tests: loop {
        let contents = opt.gen_contents().expect(
            "Couldn't get test contents. Make sure the specified language actually exists.",
        );
        if contents.is_empty() {
            println!("Test contents were empty. Exiting...");
            return Ok(());
        };

        if run_test(&config, Test::new(contents))? {
            loop {
                match event::read()? {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('r'),
                        modifiers: KeyModifiers::NONE,
                    }) => break,

                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                    })
                    | Event::Key(KeyEvent {
                        code: KeyCode::Char('c'),
                        modifiers: KeyModifiers::CONTROL,
                    }) => break 'tests,

                    _ => (),
                }
            }
        } else {
            return exit();
        }
    }
    exit()
}
