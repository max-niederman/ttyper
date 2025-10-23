/*
ttyper/src/main.rs

Main entry point for ttyper with updated save/resume/delete prompt loop.

This file was rebuilt to ensure that when the user deletes a save during the
multiple-save prompt, the program returns to the save-choices prompt (instead
of continuing to start the test). The user can delete multiple saves in a row,
choose a save to resume, create a new named save, or continue without loading.

Note: this file is the full source for the main binary.
*/

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
use ratatui::{backend::CrosstermBackend, Terminal};
use rust_embed::RustEmbed;
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead},
    num,
    path::PathBuf,
    str,
    time::Duration,
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
                    f.render_widget(config.theme.apply_to(test), f.area());
                })?;
            }
            State::Results(results) => {
                terminal.draw(|f| {
                    f.render_widget(config.theme.apply_to(results), f.area());
                })?;
            }
        }
        Ok(())
    }
}

/// Prompt user whether to resume from save file (simple yes/no)
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

    Ok(input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes"))
}

/// Prompt the user to enter a name for a new save. Returns None if the user cancels.
fn prompt_input_name() -> io::Result<Option<String>> {
    use std::io::Write;
    write!(
        io::stdout(),
        "Enter a name for the new save (leave empty to cancel): "
    )?;
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let s = input.trim().to_string();
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
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

    // Track which save file (if any) we should use for autosave / deletion.
    let mut active_save_path: Option<PathBuf> = None;

    let mut state = State::Test({
        let mut test = Test::new(contents.clone(), !opt.no_backtrack, opt.sudden_death);
        test.scroll_mode = opt.scroll_mode;

        // Check for save file(s) and prompt user to resume
        if let (Some(save_mgr), Some(file_path)) = (&save_manager, input_file_path) {
            // Load all saves related to this file (sorted oldest -> newest)
            let mut saves = save_mgr.load_all_save_states(file_path).unwrap_or_default();

            if !saves.is_empty() {
                let mut chosen_save_path: Option<PathBuf> = None;

                if !opt.no_resume_prompt {
                    // Temporarily restore terminal to show prompt
                    terminal::disable_raw_mode()?;
                    execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen,)?;

                    // Loop the prompt so that deletion returns to the choices instead of starting the test.
                    loop {
                        // If all saves have been deleted, stop prompting.
                        if saves.is_empty() {
                            break;
                        }

                        if saves.len() == 1 {
                            // Single save: behave like previous prompt, but offer to create a new-named save when the user says no.
                            let (ref path, ref state_saved) = &saves[0];
                            let resume = prompt_resume(path)?;
                            if resume {
                                state_saved.apply_to_test(&mut test);
                                chosen_save_path = Some(path.clone());
                                break;
                            } else {
                                // Ask if the user would like to save under a new name instead of overwriting
                                use std::io::Write;
                                write!(
                                    io::stdout(),
                                    "\nWould you like to save your progress under a new name? (y/N): "
                                )?;
                                io::stdout().flush()?;
                                let mut yn = String::new();
                                io::stdin().read_line(&mut yn)?;
                                if yn.trim().eq_ignore_ascii_case("y")
                                    || yn.trim().eq_ignore_ascii_case("yes")
                                {
                                    if let Some(name) = prompt_input_name()? {
                                        let new_path =
                                            save_mgr.save_test_to_name(&test, file_path, &name)?;
                                        chosen_save_path = Some(new_path);
                                        writeln!(
                                            io::stdout(),
                                            "Saved progress to {}",
                                            chosen_save_path
                                                .as_ref()
                                                .and_then(|p| p.file_name())
                                                .map(|n| n.to_string_lossy())
                                                .unwrap_or_else(|| std::borrow::Cow::Borrowed(
                                                    "<unknown>"
                                                ))
                                        )?;
                                    }
                                }
                                // After this single-save branch, regardless of save/no-save,
                                // return to calling code (do not loop infinitely on single save).
                                break;
                            }
                        } else {
                            // Multiple saves: let the user pick one, create a new named save, delete saves, or skip.
                            writeln!(
                                io::stdout(),
                                "Found multiple saved progress files for {}:",
                                file_path
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("<unknown>")
                            )?;
                            for (i, (p, s)) in saves.iter().enumerate() {
                                let name = p
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("<unknown>");
                                writeln!(
                                    io::stdout(),
                                    "  {}) {} (saved at {})",
                                    i + 1,
                                    name,
                                    s.timestamp
                                )?;
                            }
                            use std::io::Write;
                            write!(
                                io::stdout(),
                                "Enter a number to resume, 'n' to create a new save, 'dN' to delete save N (e.g. d2), or Enter to continue without loading: "
                            )?;
                            io::stdout().flush()?;
                            let mut input = String::new();
                            io::stdin().read_line(&mut input)?;
                            let trimmed = input.trim();
                            if trimmed.is_empty() {
                                // user chose to continue without loading
                                break;
                            } else if trimmed.eq_ignore_ascii_case("n") {
                                if let Some(name) = prompt_input_name()? {
                                    let new_path =
                                        save_mgr.save_test_to_name(&test, file_path, &name)?;
                                    chosen_save_path = Some(new_path);
                                    writeln!(
                                        io::stdout(),
                                        "Saved progress to {}",
                                        chosen_save_path
                                            .as_ref()
                                            .and_then(|p| p.file_name())
                                            .map(|n| n.to_string_lossy())
                                            .unwrap_or_else(|| std::borrow::Cow::Borrowed(
                                                "<unknown>"
                                            ))
                                    )?;
                                    break;
                                } else {
                                    // user canceled naming; loop back to choices
                                    continue;
                                }
                            } else if trimmed.starts_with('d') || trimmed.starts_with('D') {
                                // Attempt to parse a deletion command like "d2" or "d 2"
                                let rest = trimmed[1..].trim();
                                if let Ok(idx) = rest.parse::<usize>() {
                                    if idx >= 1 && idx <= saves.len() {
                                        let (ref path, _) = &saves[idx - 1];
                                        // Confirm deletion
                                        use std::io::Write;
                                        write!(
                                            io::stdout(),
                                            "Delete save {}? (y/N): ",
                                            path.file_name()
                                                .map(|n| n.to_string_lossy())
                                                .unwrap_or_else(|| std::borrow::Cow::Borrowed(
                                                    "<unknown>"
                                                ))
                                        )?;
                                        io::stdout().flush()?;
                                        let mut conf = String::new();
                                        io::stdin().read_line(&mut conf)?;
                                        if conf.trim().eq_ignore_ascii_case("y")
                                            || conf.trim().eq_ignore_ascii_case("yes")
                                        {
                                            let _ = save_mgr.delete_save_by_path(path);
                                            // remove from the local list so subsequent loop sees updated list
                                            saves.remove(idx - 1);
                                            writeln!(io::stdout(), "Deleted save.")?;
                                            // Return to top of loop and re-display remaining saves.
                                            continue;
                                        } else {
                                            // user canceled deletion; re-display choices
                                            continue;
                                        }
                                    } else {
                                        // invalid index; re-display list
                                        writeln!(io::stdout(), "Invalid save number.")?;
                                        continue;
                                    }
                                } else {
                                    // couldn't parse number; re-display list
                                    writeln!(io::stdout(), "Invalid delete command.")?;
                                    continue;
                                }
                            } else if let Ok(idx) = trimmed.parse::<usize>() {
                                if idx >= 1 && idx <= saves.len() {
                                    let (ref path, ref state_saved) = &saves[idx - 1];
                                    state_saved.apply_to_test(&mut test);
                                    chosen_save_path = Some(path.clone());
                                    break;
                                } else {
                                    writeln!(io::stdout(), "Invalid save number.")?;
                                    continue;
                                }
                            } else {
                                // unknown input; re-display
                                writeln!(io::stdout(), "Unrecognized input.")?;
                                continue;
                            }
                        }
                    } // end loop

                    // Re-enable raw mode and alternate screen
                    terminal::enable_raw_mode()?;
                    execute!(
                        io::stdout(),
                        cursor::Hide,
                        cursor::SavePosition,
                        terminal::EnterAlternateScreen,
                    )?;
                    terminal.clear()?;
                } else {
                    // No prompt: auto-resume using the most recent save
                    let last = saves.last().unwrap();
                    last.1.apply_to_test(&mut test);
                    chosen_save_path = Some(last.0.clone());
                };

                if let Some(p) = chosen_save_path {
                    active_save_path = Some(p);
                }
            }
        }

        test
    });

    state.render_into(&mut terminal, &config)?;
    let mut last_save_time = std::time::Instant::now();
    let save_interval = std::time::Duration::from_secs(5); // Autosave every 5 seconds

    loop {
        if let State::Test(ref mut test) = state {
            test.update_duration();
        }

        let timeout = Duration::from_millis(1000);
        if event::poll(timeout)? {
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
                    State::Test(ref mut test) => {
                        if let (Some(save_mgr), Some(file_path)) = (&save_manager, input_file_path)
                        {
                            if let Some(ref chosen) = active_save_path {
                                let _ = save_mgr.save_test_to_path(test, file_path, chosen);
                            } else {
                                let _ = save_mgr.save_test(test, file_path);
                            }
                        }
                        test.set_timer_active(false);
                        state = State::Results(Results::from(&*test));
                    }
                    State::Results(_) => break,
                },
                _ => {}
            }

            match state {
                State::Test(ref mut test) => {
                    if let Event::Key(key) = event {
                        test.set_timer_active(true);
                        test.handle_key(key);

                        // Autosave progress
                        if let Some(save_mgr) = &save_manager {
                            let now = std::time::Instant::now();
                            if now.duration_since(last_save_time) >= save_interval {
                                if let Some(file_path) = input_file_path {
                                    if let Some(ref chosen) = active_save_path {
                                        // Save to the chosen save file (do not overwrite other saves)
                                        let _ = save_mgr.save_test_to_path(test, file_path, chosen);
                                    } else {
                                        // Default behaviour: overwrite single default save file
                                        let _ = save_mgr.save_test(test, file_path);
                                    }
                                }
                                last_save_time = now;
                            }
                        }

                        if test.complete {
                            test.set_timer_active(false);
                            // Delete save file when test is completed
                            if let Some(save_mgr) = &save_manager {
                                if let Some(file_path) = input_file_path {
                                    if let Some(ref chosen) = active_save_path {
                                        // Delete the specific save file that was used
                                        let _ = save_mgr.delete_save_by_path(chosen);
                                    } else {
                                        // Default behaviour: delete all saves related to this file
                                        let _ = save_mgr.delete_save(file_path);
                                    }
                                }
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
                        let mut new_test =
                            Test::new(new_contents, !opt.no_backtrack, opt.sudden_death);
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
        } else {
            if let State::Test(ref mut test) = state {
                test.set_timer_active(false);
            }
        }

        state.render_into(&mut terminal, &config)?;
    }

    // Save final state before exiting if we're in the middle of a test
    if let (Some(save_mgr), Some(file_path), State::Test(ref test)) =
        (&save_manager, input_file_path, &state)
    {
        if test.current_word > 0 {
            if let Some(ref chosen) = active_save_path {
                let _ = save_mgr.save_test_to_path(test, file_path, chosen);
            } else {
                let _ = save_mgr.save_test(test, file_path);
            }
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
