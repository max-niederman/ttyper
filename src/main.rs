mod config;
mod test;
mod ui;

use config::Config;
use test::{results::Results, DisplayLine, Test};

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use crossterm::{
    self, cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, terminal,
};
use rand::{seq::SliceRandom, thread_rng};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    terminal::Terminal,
};
use rust_embed::RustEmbed;
use std::{
    ffi::OsString,
    fs,
    io::{self, BufRead, Read},
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

    /// Disable backspace
    #[arg(long)]
    no_backspace: bool,

    /// Treat input as raw text: split into words and preserve line layout
    #[arg(long)]
    raw: bool,

    /// Display all UTF-8 but skip non-ASCII-printable characters during typing
    #[arg(long)]
    qwerty: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}

impl Opt {
    /// Generate test contents.
    ///
    /// Returns `(words, lines)` where `lines` describes the original file
    /// layout (empty for language/word-list mode).
    fn gen_contents(&self) -> Option<(Vec<String>, Vec<DisplayLine>)> {
        match &self.contents {
            Some(path) => {
                if self.raw {
                    // Raw mode: split into words, preserve line structure
                    // including empty lines and indentation.
                    let text = if path.as_os_str() == "-" {
                        let mut buf = String::new();
                        std::io::stdin()
                            .lock()
                            .read_to_string(&mut buf)
                            .expect("Error reading from stdin.");
                        buf
                    } else {
                        fs::read_to_string(path).expect("Error reading file.")
                    };

                    let mut words = Vec::new();
                    let mut lines = Vec::new();

                    for line in text.lines() {
                        // Capture leading whitespace, expand tabs to 4 spaces
                        let indent: String = line
                            .chars()
                            .take_while(|c| c.is_whitespace())
                            .collect::<String>()
                            .replace('\t', "    ");

                        let word_start = words.len();
                        for token in line.split_whitespace() {
                            // Strip control characters, keep all printable chars
                            let word: String =
                                token.chars().filter(|c| !c.is_control()).collect();
                            if !word.is_empty() {
                                words.push(word);
                            }
                        }
                        let word_count = words.len() - word_start;

                        lines.push(DisplayLine {
                            indent,
                            word_start,
                            word_count,
                        });
                    }

                    Some((words, lines))
                } else {
                    // Default: each line is one word (original behavior).
                    let lines: Vec<String> = if path.as_os_str() == "-" {
                        std::io::stdin()
                            .lock()
                            .lines()
                            .map_while(Result::ok)
                            .collect()
                    } else {
                        let file =
                            fs::File::open(path).expect("Error reading language file.");
                        io::BufReader::new(file)
                            .lines()
                            .map_while(Result::ok)
                            .collect()
                    };

                    Some((lines, Vec::new()))
                }
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

                Some((contents, Vec::new()))
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
                    let area = f.size();
                    f.render_widget(config.theme.apply_to(test), area);

                    // Position cursor at end of input for IME composition support
                    let prompt_constraint = if !test.lines.is_empty() {
                        Constraint::Min(6)
                    } else {
                        Constraint::Length(6)
                    };
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(3), prompt_constraint])
                        .split(area);
                    let inner_x = chunks[0].x + 1;
                    let inner_y = chunks[0].y + 1;
                    let progress_width =
                        ratatui::text::Line::from(test.words[test.current_word].progress.as_str())
                            .width() as u16;
                    let max_cursor_x = chunks[0].right().saturating_sub(2);
                    f.set_cursor((inner_x + progress_width).min(max_cursor_x), inner_y);
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

fn main() -> io::Result<()> {
    let opt = Opt::parse();
    if opt.debug {
        dbg!(&opt);
    }

    let config = opt.config();
    if opt.debug {
        dbg!(&config);
    }

    if let Some(Command::Completions { shell }) = opt.command {
        generate(shell, &mut Opt::command(), "ttyper", &mut io::stdout());
        return Ok(());
    }

    if opt.list_languages {
        opt.languages()
            .unwrap()
            .for_each(|name| println!("{}", name.to_str().expect("Ill-formatted language name.")));

        return Ok(());
    }

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    let (contents, lines) = opt
        .gen_contents()
        .expect("Couldn't get test contents. Make sure the specified language actually exists.");

    if contents.is_empty() {
        eprintln!("Error: the provided file or language contains no words to type.");
        eprintln!("If you specified a file, make sure it isn't empty.");
        std::process::exit(1);
    }

    terminal::enable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::Hide,
        cursor::SavePosition,
        terminal::EnterAlternateScreen,
    )?;
    terminal.clear()?;

    let mut state = State::Test(Test::new(
        contents,
        !opt.no_backtrack,
        opt.sudden_death,
        !opt.no_backspace,
        lines,
        opt.qwerty,
    ));

    state.render_into(&mut terminal, &config)?;
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
                    if test.complete {
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
                    let (new_contents, new_lines) = opt.gen_contents().expect(
                        "Couldn't get test contents. Make sure the specified language actually exists.",
                    );
                    if new_contents.is_empty() {
                        continue;
                    }
                    state = State::Test(Test::new(
                        new_contents,
                        !opt.no_backtrack,
                        opt.sudden_death,
                        !opt.no_backspace,
                        new_lines,
                        opt.qwerty,
                    ));
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
                    state = State::Test(Test::new(
                        practice_words,
                        !opt.no_backtrack,
                        opt.sudden_death,
                        !opt.no_backspace,
                        Vec::new(),
                        opt.qwerty,
                    ));
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

    terminal::disable_raw_mode()?;
    execute!(
        io::stdout(),
        cursor::RestorePosition,
        cursor::Show,
        terminal::LeaveAlternateScreen,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn make_opt(path: PathBuf, raw: bool, qwerty: bool) -> Opt {
        Opt {
            contents: Some(path),
            debug: false,
            words: num::NonZeroUsize::new(50).unwrap(),
            config: None,
            language_file: None,
            language: None,
            list_languages: false,
            no_backtrack: false,
            sudden_death: false,
            no_backspace: false,
            raw,
            qwerty,
            command: None,
        }
    }

    #[test]
    fn gen_contents_empty_file_returns_empty_vec() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        fs::File::create(&path).unwrap();

        let (contents, _) = make_opt(path, false, false).gen_contents().unwrap();
        assert!(contents.is_empty(), "empty file should produce empty vec");
    }

    #[test]
    fn gen_contents_default_lines_as_words() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("words.txt");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "hello world rust").unwrap();

        let (contents, lines) = make_opt(path, false, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello world rust"]);
        assert!(lines.is_empty());
    }

    #[test]
    fn gen_contents_raw_splits_words() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("words.txt");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "hello world rust").unwrap();

        let (contents, lines) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello", "world", "rust"]);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].word_start, 0);
        assert_eq!(lines[0].word_count, 3);
    }

    #[test]
    fn gen_contents_raw_preserves_unicode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("unicode.txt");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "hello\u{2014}world \u{201c}quoted\u{201d}").unwrap();

        let (contents, _) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello\u{2014}world", "\u{201c}quoted\u{201d}"]);
    }

    #[test]
    fn gen_contents_raw_multiline_tracks_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "first line\nsecond line\n\nfourth line").unwrap();

        let (contents, lines) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(
            contents,
            vec!["first", "line", "second", "line", "fourth", "line"]
        );
        // 4 lines: line 1, line 2, empty line, line 4
        assert_eq!(lines.len(), 4);
        assert_eq!((lines[0].word_start, lines[0].word_count), (0, 2));
        assert_eq!((lines[1].word_start, lines[1].word_count), (2, 2));
        assert_eq!(lines[2].word_count, 0); // empty line preserved
        assert_eq!((lines[3].word_start, lines[3].word_count), (4, 2));
    }

    #[test]
    fn gen_contents_raw_empty_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("empty.txt");
        fs::File::create(&path).unwrap();

        let (contents, lines) = make_opt(path, true, false).gen_contents().unwrap();
        assert!(contents.is_empty());
        assert!(lines.is_empty());
    }

    #[test]
    fn gen_contents_raw_preserves_whitespace_only_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("spaces.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "hello\n   \n  \t  \nworld").unwrap();

        let (contents, lines) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello", "world"]);
        // 4 lines total: "hello", whitespace-only, whitespace-only, "world"
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].word_count, 1);
        assert_eq!(lines[1].word_count, 0); // whitespace-only preserved
        assert_eq!(lines[2].word_count, 0);
        assert_eq!(lines[3].word_count, 1);
    }

    #[test]
    fn gen_contents_raw_keeps_all_unicode_tokens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("alluni.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "hello \u{2014}\u{2014}\u{2014} world").unwrap();

        let (contents, _) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello", "\u{2014}\u{2014}\u{2014}", "world"]);
    }

    #[test]
    fn gen_contents_default_multiline_each_line_is_word() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("multi.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "first line\nsecond line").unwrap();

        let (contents, lines) = make_opt(path, false, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["first line", "second line"]);
        assert!(lines.is_empty());
    }

    #[test]
    fn gen_contents_raw_preserves_punctuation() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("punct.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "it's a \"test\" (100%); done!").unwrap();

        let (contents, _) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["it's", "a", "\"test\"", "(100%);", "done!"]);
    }

    #[test]
    fn gen_contents_raw_expands_tabs_in_indent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tabs.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "hello\n\tindented\n\t\tdeep").unwrap();

        let (contents, lines) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello", "indented", "deep"]);
        assert_eq!(lines[0].indent, "");
        assert_eq!(lines[1].indent, "    ");      // 1 tab = 4 spaces
        assert_eq!(lines[2].indent, "        ");  // 2 tabs = 8 spaces
    }

    #[test]
    fn gen_contents_raw_strips_control_chars_from_words() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ctrl.txt");
        let mut f = fs::File::create(&path).unwrap();
        write!(f, "hel\x07lo wor\x00ld").unwrap();

        let (contents, _) = make_opt(path, true, false).gen_contents().unwrap();
        assert_eq!(contents, vec!["hello", "world"]);
    }
}
