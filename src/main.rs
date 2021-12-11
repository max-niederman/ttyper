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
use std::time::SystemTime;

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

    #[structopt(short, long, default_value = "3")]
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
    let results = Results::from(&test);
    terminal.clear()?;
    terminal.draw(|f| {
        f.render_widget(&results, f.size());
    })?;
    
    let csv = Pumper {results_obj: results,
                    test_obj: test,
                    args: String::from("foo")}; 
    
    let a = csv.return_csv();
    println!("{}",a);
    
    Ok(true)
}

/*
adding codes likes this in the main file is pretty problematic, i can try to mess around with the `mod` thing but I am not a rust pro,
so i am not really sure what i am doing is right or not

header
datetime    settings    results.cps.overall results.accuracy.overall
xx          yy          aa                  bb              


is is possible to implement linkedhashmap for results::..::per_key so chronological data could be read easier?

data    character (hopefully it is because it's linked) time elapsed since last      correct? (Some<T>)
index   words[index].event[index].key.code.(key=0)    results.cps.per_event[index] words[index].event[index].correct
1       aa                                              yy                             kk                   
*/

pub struct Pumper {
    results_obj: Results,
    test_obj: Test,
    args: String,
} 

impl Pumper {
    fn gen_datetime(&self) -> u64 {
        let mut a: u64 = 0;
        match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {a = n.as_secs()},
            Err(_) => (),
        }
        return a;
    }

    fn gen_settings(&self) -> String {
        //TODO: do it
        String::new()
    }

    fn gen_csv_from_vec(&self, v: Vec<String>) -> String {
        let s = v.into_iter().map(|i| i.to_string()+", ").collect::<String>();
        return s;
    }

    // it can be done without these things by doing the string directly instead of making string to vector and make back to string
    //  but at least it is easier to manage for the time being
    fn unpack_header(&self) -> Vec<String> {
        let mut ret: Vec<String> = Vec::new();

        // i am not sure if this is the rust way?
        ret.push(self.gen_datetime().to_string());
        ret.push(self.gen_settings().to_string());
        ret.push(self.results_obj.cps.overall.to_string());
        ret.push(self.results_obj.accuracy.overall.to_string());

        return ret;
    }

    fn unpack_data(&self) -> Vec<Vec<String>> {
        let mut ret: Vec<Vec<String>> = Vec::new();

        // is this the rust way? i dont even know, there are too many functions and i dont know how to use them
        // and again it's faster to do this directly instead of making string into vector then string again
        // but i think rust is fast so should it be fine?
        let mut true_index = 0;
        for index in 0..self.test_obj.words.len() {
            let curr_word = &self.test_obj.words[index];

            for index2 in 0..curr_word.events.len() {
                let curr_char = &curr_word.events[index2];
                let mut curr_vec: Vec<String> = Vec::new();

                curr_vec.push((true_index).to_string());

                // i really dont know what i am doing
                if let KeyCode::Char(character) = curr_char.key.code {
                    curr_vec.push(character.to_string());
                }
                curr_vec.push(self.results_obj.cps.per_event[true_index].to_string());
                curr_vec.push(curr_char.correct.is_some().to_string());

                ret.push(curr_vec);
                true_index+=1;
            }
        }
        return ret;
    }

    fn return_csv(&self) -> String {
        let mut ret: String = String::new();

        ret.push_str(&self.gen_csv_from_vec(self.unpack_header()));
        ret.push_str(&String::from("\n"));

        for s in self.unpack_data().iter() {
            // is this memory safe????
            ret.push_str(&self.gen_csv_from_vec(s.to_vec()));
            ret.push_str(&String::from("\n"));
        }

        return ret;
    }
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
