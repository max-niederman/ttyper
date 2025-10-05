mod config;
mod save;
mod test;
mod ui;

use config::Config;
use save::SaveManager;
use test::{results::Results, Test};

use clap::Parser;
use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal,
};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::{backend::CrosstermBackend, terminal::Terminal};
use rust_embed::RustEmbed;
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead},
    num,
    path::PathBuf,
    str,
};

#[derive(RustEmbed)]
#[folder = "resources/runtime"]
struct Resources;

#[derive(Debug, Parser)]
#[command(about, version)]
struct Opt {
    /// Read test contents from the specified file, or "-" for stdin
    #[arg(value_name = "PATH")]
    contents: Option<PathBuf>,

    #[arg(short, long)]
    debug: bool,

    /// Specify word count
    #[arg(short, long, value_name = "N", default_value = "50")]
    words: num::NonZeroUsize,

    /// Use config file
    #[arg(short, long, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Specify test language in file
    #[arg(long, value_name = "PATH")]
    language_file: Option<PathBuf>,

    /// Specify test language
    #[arg(short, long, value_name = "LANG")]
    language: Option<String>,

    /// List installed languages
    #[arg(long)]
    list_languages: bool,

    /// Disable backtracking to completed words
    #[arg(long)]
    no_backtrack: bool,

    /// Enable sudden death mode to restart on first error
    #[arg(long)]
    sudden_death: bool,

    /// Keep current word at top of display (useful for long texts)
    #[arg(long)]
    scroll_mode: bool,

    /// Enable autosave to resume typing progress
    #[arg(long)]
    autosave: bool,

    /// Disable the prompt to resume from save file
    #[arg(long)]
    no_resume_prompt: bool,
}

impl Opt {
    fn gen_contents(&self) -> Option<Vec<String>> {
        match &self.contents {
            Some(path) => {
                let lines: Vec<String> = if path.as_os_str() == "-" {
                    std::io::stdin()
                        .lock()
                        .lines()
                        .map_while(Result::ok)
                        .collect()
                } else {
                    let file = fs::File::open(path).expect("Error reading language file.");
                    io::BufReader::new(file)
                        .lines()
                        .map_while(Result::ok)
                        .collect()
                };

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
                    .and_then(Result::ok)
                    .or_else(|| fs::read(self.language_dir().join(&lang_name)).ok())
                    .or_else(|| {
                        Resources::get(&format!("language/{}", &lang_name))
                            .map(|f| f.data.into_owned())
                    })?;

                let mut rng = thread_rng();

                let mut language: Vec<&str> = str::from_utf8(&bytes)
                    .expect("Language file had non-utf8 encoding.")
                    .lines()
                    .collect();
                language.shuffle(&mut rng);

                let mut contents: Vec<_> = language
                    .into_iter()
                    .cycle()
                    .take(self.words.get())
                    .map(ToOwned::to_owned)
                    .collect();
                contents.shuffle(&mut rng);

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
        .map(|bytes| {
            toml::from_str(str::from_utf8(&bytes).unwrap_or_default())
                .expect("Configuration was ill-formed.")
        })
        .unwrap_or_default()
    }

    /// Installed languages under config directory
    fn languages(&self) -> io::Result<impl Iterator<Item = OsString>> {
        let builtin = Resources::iter().filter_map(|name| {
            name.strip_prefix("language/")
                .map(ToOwned::to_owned)
                .map(OsString::from)
        });

        let configured = self
            .language_dir()
            .read_dir()
            .into_iter()
            .flatten()
            .map_while(Result::ok)
            .map(|e| e.file_name());

        Ok(builtin.chain(configured))
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

enum State {
    Test(Test),
    Results(Results),
}

impl State {
    fn render_into<B: ratatui::backend::Backend>(
        &self,
        terminal: &mut Terminal<B>,
        config: &Config,
    ) -> io::Result<()> {
        match self {
            State::Test(test) => {
                terminal.draw(|f| {
                    f.render_widget(config.theme.apply_to(test), f.size());
                })?;
            }
            State::Results(results) => {
                terminal.draw(|f| {
                    f.render_widget(config.theme.apply_to(results), f.size());
                })?;
            }
        }
        Ok(())
    }
}

/// Prompt user whether to resume from save file
fn prompt_resume(file_path: &std::path::Path) -> io::Result<bool> {
    execute!(
        io::stdout(),
        Print(format!(
            "Found saved progress for {:?}\n",
            file_path.file_name().unwrap_or_default()
        )),
        Print("Do you want to resume from where you left off? (y/N): ")
    )?;

    // Flush stdout to ensure prompt is displayed
    use std::io::Write;
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
}

fn main() -> io::Result<()> {
    let opt = Opt::parse();
    if opt.debug {
        dbg!(&opt);
    }

    let config = opt.config();
    if opt.debug {
        dbg!(&config);
    }

    if opt.list_languages {
        opt.languages()
            .unwrap()
            .for_each(|name| println!("{}", name.to_str().expect("Ill-formatted language name.")));

        return Ok(());
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let contents = opt
        .gen_contents()
        .expect("Couldn't get test contents. Make sure the specified language actually exists.");

    let save_manager = if opt.autosave {
        if opt.debug {
            println!("Autosave enabled - progress will be saved automatically");
        }
        Some(SaveManager::new()?)
    } else {
        None
    };

    let input_file_path = opt.contents.as_ref();

    terminal::enable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::Hide,
        cursor::SavePosition,
        terminal::EnterAlternateScreen,
    )?;
    terminal.clear()?;

    let mut state = State::Test({
        let mut test = Test::new(contents.clone(), !opt.no_backtrack, opt.sudden_death);
        test.scroll_mode = opt.scroll_mode;

        // Check for save file and prompt user to resume
        if let (Some(save_mgr), Some(file_path)) = (&save_manager, input_file_path) {
            if let Some(save_state) = save_mgr.load_save_state(file_path) {
                let should_resume = if !opt.no_resume_prompt {
                    // Temporarily restore terminal to show prompt
                    terminal::disable_raw_mode()?;
                    execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen,)?;

                    let resume = prompt_resume(file_path)?;

                    // Re-enable raw mode and alternate screen
                    terminal::enable_raw_mode()?;
                    execute!(
                        io::stdout(),
                        cursor::Hide,
                        cursor::SavePosition,
                        terminal::EnterAlternateScreen,
                    )?;
                    terminal.clear()?;
                    resume
                } else {
                    true
                };

                if should_resume {
                    save_state.apply_to_test(&mut test);
                }
            }
        }

        test
    });

    state.render_into(&mut terminal, &config)?;
    let mut last_save_time = std::time::Instant::now();
    let save_interval = std::time::Duration::from_secs(5); // Autosave every 5 seconds

    loop {
        let event = event::read()?;

        // handle exit controls
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::CONTROL,
                ..
            }) => break,
            Event::Key(KeyEvent {
                code: KeyCode::Esc,
                kind: KeyEventKind::Press,
                modifiers: KeyModifiers::NONE,
                ..
            }) => match state {
                State::Test(ref test) => {
                    state = State::Results(Results::from(test));
                }
                State::Results(_) => break,
            },
            _ => {}
        }

        match state {
            State::Test(ref mut test) => {
                if let Event::Key(key) = event {
                    test.handle_key(key);

                    // Autosave progress
                    if let (Some(save_mgr), Some(file_path)) = (&save_manager, input_file_path) {
                        let now = std::time::Instant::now();
                        if now.duration_since(last_save_time) >= save_interval {
                            let _ = save_mgr.save_test(test, file_path);
                            last_save_time = now;
                        }
                    }

                    if test.complete {
                        // Delete save file when test is completed
                        if let (Some(save_mgr), Some(file_path)) = (&save_manager, input_file_path)
                        {
                            let _ = save_mgr.delete_save(file_path);
                        }
                        state = State::Results(Results::from(&*test));
                    }
                }
            }
            State::Results(ref result) => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('r'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    let new_contents = opt.gen_contents().expect(
                        "Couldn't get test contents. Make sure the specified language actually exists.",
                    );
                    let mut new_test = Test::new(new_contents, !opt.no_backtrack, opt.sudden_death);
                    new_test.scroll_mode = opt.scroll_mode;
                    state = State::Test(new_test);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    if result.missed_words.is_empty() {
                        continue;
                    }
                    // repeat each missed word 5 times
                    let mut practice_words: Vec<String> = (result.missed_words)
                        .iter()
                        .flat_map(|w| vec![w.clone(); 5])
                        .collect();
                    practice_words.shuffle(&mut thread_rng());
                    let mut practice_test =
                        Test::new(practice_words, !opt.no_backtrack, opt.sudden_death);
                    practice_test.scroll_mode = opt.scroll_mode;
                    state = State::Test(practice_test);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    kind: KeyEventKind::Press,
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => break,
                _ => {}
            },
        }

        state.render_into(&mut terminal, &config)?;
    }

    // Save final state before exiting if we're in the middle of a test
    if let (Some(save_mgr), Some(file_path), State::Test(ref test)) =
        (&save_manager, input_file_path, &state)
    {
        if !test.complete && test.current_word > 0 {
            let _ = save_mgr.save_test(test, file_path);
        }
    }

    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::RestorePosition,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )?;

    Ok(())
}
