use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::test::Test;

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveState {
    pub file_path: PathBuf,
    pub file_hash: String,
    pub current_word: usize,
    pub words: Vec<SavedWord>,
    pub timestamp: u64,
    pub backtracking_enabled: bool,
    pub sudden_death_enabled: bool,
    pub scroll_mode: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedWord {
    pub text: String,
    pub progress: String,
}

impl SaveState {
    /// Create a new SaveState from a Test and file path
    pub fn from_test(test: &Test, file_path: &Path) -> io::Result<Self> {
        let file_hash = Self::calculate_file_hash(file_path)?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let words = test
            .words
            .iter()
            .map(|word| SavedWord {
                text: word.text.clone(),
                progress: word.progress.clone(),
            })
            .collect();

        Ok(SaveState {
            file_path: file_path.to_path_buf(),
            file_hash,
            current_word: test.current_word,
            words,
            timestamp,
            backtracking_enabled: test.backtracking_enabled,
            sudden_death_enabled: test.sudden_death_enabled,
            scroll_mode: test.scroll_mode,
        })
    }

    /// Apply saved state to a Test
    pub fn apply_to_test(&self, test: &mut Test) {
        test.current_word = self.current_word;
        test.backtracking_enabled = self.backtracking_enabled;
        test.sudden_death_enabled = self.sudden_death_enabled;
        test.scroll_mode = self.scroll_mode;

        // Apply saved progress to words
        for (i, saved_word) in self.words.iter().enumerate() {
            if i < test.words.len() && test.words[i].text == saved_word.text {
                test.words[i].progress = saved_word.progress.clone();
            }
        }
    }

    /// Calculate a simple hash of file contents for verification
    fn calculate_file_hash(file_path: &Path) -> io::Result<String> {
        let contents = fs::read_to_string(file_path)?;
        // Simple hash using content length and first/last chars
        let hash = format!(
            "{}:{}:{}",
            contents.len(),
            contents.chars().next().unwrap_or(' ') as u32,
            contents.chars().last().unwrap_or(' ') as u32
        );
        Ok(hash)
    }

    // /// Check if the saved state is still valid for the given file
    // pub fn is_valid_for_file(&self, file_path: &Path) -> bool {
    //     if let Ok(current_hash) = Self::calculate_file_hash(file_path) {
    //         self.file_hash == current_hash && self.file_path == file_path
    //     } else {
    //         false
    //     }
    // }
}

pub struct SaveManager {
    save_dir: PathBuf,
}

impl SaveManager {
    pub fn new() -> io::Result<Self> {
        let save_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?
            .join("ttyper")
            .join("saves");

        // Create save directory if it doesn't exist
        fs::create_dir_all(&save_dir)?;

        Ok(SaveManager { save_dir })
    }

    /// Sanitize a string to be safe for filenames
    fn sanitize_name(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                other => other,
            })
            .collect()
    }

    /// Get the default save file path for a given input file (backwards compatible)
    fn get_save_file_path(&self, file_path: &Path) -> PathBuf {
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");

        // Create a safe filename by replacing path separators
        let safe_name = format!(
            "{}_{}.json",
            file_name.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_"),
            file_path
                .to_string_lossy()
                .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_")
        );

        self.save_dir.join(safe_name)
    }

    /// Get a save file path for a given input file with a custom name.
    /// This allows creating alternative save files when the user declines to resume.
    fn get_save_file_path_with_name(&self, file_path: &Path, name: &str) -> PathBuf {
        let base = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let safe_base = Self::sanitize_name(base);
        let safe_name = Self::sanitize_name(name);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let filename = format!("{}_{}_{}.json", safe_base, safe_name, timestamp);
        self.save_dir.join(filename)
    }

    /// Save the current test state to the default save path (backwards compatible)
    pub fn save_test(&self, test: &Test, file_path: &Path) -> io::Result<PathBuf> {
        let save_state = SaveState::from_test(test, file_path)?;
        let save_file_path = self.get_save_file_path(file_path);

        let json = serde_json::to_string_pretty(&save_state)?;
        fs::write(&save_file_path, json)?;

        Ok(save_file_path)
    }

    /// Save the current test state to a specific named save file.
    /// Returns the path written to so callers can inform the user.
    pub fn save_test_to_name(
        &self,
        test: &Test,
        file_path: &Path,
        name: &str,
    ) -> io::Result<PathBuf> {
        let save_state = SaveState::from_test(test, file_path)?;
        let save_file_path = self.get_save_file_path_with_name(file_path, name);

        let json = serde_json::to_string_pretty(&save_state)?;
        fs::write(&save_file_path, json)?;

        Ok(save_file_path)
    }

    /// List all save files in the saves directory that reference `file_path` and are parseable.
    /// Returns a vector of (save_file_path, SaveState).
    pub fn list_save_states(&self, file_path: &Path) -> io::Result<Vec<(PathBuf, SaveState)>> {
        let mut results = Vec::new();
        let entries = fs::read_dir(&self.save_dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }

            if let Ok(json) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str::<SaveState>(&json) {
                    // Only return saves that match the target file path (exact)
                    if state.file_path == file_path {
                        results.push((path, state));
                    }
                } else {
                    // If the file cannot be parsed, try removing it (cleanup)
                    let _ = fs::remove_file(&path);
                }
            }
        }

        Ok(results)
    }

    // /// Load the most appropriate save state for `file_path`.
    // /// If multiple saves are present, returns the one with the newest timestamp.
    // /// Invalid/unparseable saves are cleaned up.
    // pub fn load_save_state(&self, file_path: &Path) -> Option<SaveState> {
    //     match self.list_save_states(file_path) {
    //         Ok(mut list) if !list.is_empty() => {
    //             // Choose the SaveState with the largest timestamp (most recent)
    //             list.sort_by_key(|(_, s)| s.timestamp);
    //             let (_path, state) = list.pop().unwrap();
    //             Some(state)
    //         }
    //         _ => None,
    //     }
    // }

    /// Load all valid save states for a given file path.
    /// This lets callers (e.g. the UI) present choices to the user when multiple saves exist.
    pub fn load_all_save_states(&self, file_path: &Path) -> io::Result<Vec<(PathBuf, SaveState)>> {
        let mut states = self.list_save_states(file_path)?;
        // sort oldest -> newest
        states.sort_by_key(|(_, s)| s.timestamp);
        Ok(states)
    }

    /// Delete save files for a given input file (deletes all matching saves).
    pub fn delete_save(&self, file_path: &Path) -> io::Result<()> {
        let saves = self.list_save_states(file_path)?;
        for (path, _) in saves {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }
        Ok(())
    }

    /// Save the current test state to an explicit path (useful when user selected a specific save file)
    pub fn save_test_to_path(
        &self,
        test: &Test,
        file_path: &Path,
        save_file_path: &Path,
    ) -> io::Result<()> {
        let save_state = SaveState::from_test(test, file_path)?;
        let json = serde_json::to_string_pretty(&save_state)?;
        fs::write(save_file_path, json)?;
        Ok(())
    }

    /// Delete a specific save file by path
    pub fn delete_save_by_path(&self, save_file_path: &Path) -> io::Result<()> {
        if save_file_path.exists() {
            fs::remove_file(save_file_path)?;
        }
        Ok(())
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SaveManager")
    }
}
