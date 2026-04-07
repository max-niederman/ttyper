use crate::cli::Opt;
use crate::config::Config;
use rand::{seq::SliceRandom, thread_rng};
use rust_embed::RustEmbed;
use std::ffi::OsString;
use std::io::BufRead;
use std::path::PathBuf;
use std::{fs, io, str};

#[derive(RustEmbed)]
#[folder = "resources/runtime"]
struct Resources;

impl Opt {
    pub fn gen_contents(&self) -> Option<Vec<String>> {
        match &self.contents {
            Some(path) => {
                let lines: Vec<String> = if path.as_os_str() == "-" {
                    std::io::stdin()
                        .lock()
                        .lines()
                        .map_while(std::result::Result::ok)
                        .collect()
                } else {
                    let file = fs::File::open(path).ok()?;
                    io::BufReader::new(file)
                        .lines()
                        .map_while(std::result::Result::ok)
                        .collect()
                };

                Some(lines)
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
                    .and_then(std::result::Result::ok)
                    .or_else(|| fs::read(self.language_dir().join(&lang_name)).ok())
                    .or_else(|| {
                        Resources::get(&format!("language/{}", &lang_name))
                            .map(|f| f.data.into_owned())
                    })?;

                let mut rng = thread_rng();

                let mut language: Vec<&str> = str::from_utf8(&bytes).ok()?.lines().collect();
                language.shuffle(&mut rng);

                let mut contents: Vec<String> = language
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
    pub fn config(&self) -> Config {
        let config_path = self
            .config
            .clone()
            .unwrap_or_else(|| self.config_dir().join("config.toml"));

        fs::read(config_path)
            .map(|bytes| {
                toml::from_str::<Config>(str::from_utf8(&bytes).unwrap_or_default())
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    /// Installed languages under config directory
    pub fn languages(&self) -> io::Result<impl Iterator<Item = OsString>> {
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
            .map_while(std::result::Result::ok)
            .map(|e: fs::DirEntry| e.file_name());

        Ok(builtin.chain(configured))
    }

    /// Config directory
    pub fn config_dir(&self) -> PathBuf {
        dir_spec::config_home()
            .map(|d| d.join("ttyper"))
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Language directory under config directory
    pub fn language_dir(&self) -> PathBuf {
        self.config_dir().join("language")
    }
}
