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

    /// Check if the saved state is still valid for the given file
    pub fn is_valid_for_file(&self, file_path: &Path) -> bool {
        if let Ok(current_hash) = Self::calculate_file_hash(file_path) {
            self.file_hash == current_hash && self.file_path == file_path
        } else {
            false
        }
    }
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

    /// Get the save file path for a given input file
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

    /// Save the current test state
    pub fn save_test(&self, test: &Test, file_path: &Path) -> io::Result<()> {
        let save_state = SaveState::from_test(test, file_path)?;
        let save_file_path = self.get_save_file_path(file_path);

        let json = serde_json::to_string_pretty(&save_state)?;
        fs::write(save_file_path, json)?;

        Ok(())
    }

    /// Load saved state for a file if it exists and is valid
    pub fn load_save_state(&self, file_path: &Path) -> Option<SaveState> {
        let save_file_path = self.get_save_file_path(file_path);

        if !save_file_path.exists() {
            return None;
        }

        let json = fs::read_to_string(save_file_path).ok()?;
        let save_state: SaveState = serde_json::from_str(&json).ok()?;

        if save_state.is_valid_for_file(file_path) {
            Some(save_state)
        } else {
            // Remove invalid save file
            let _ = fs::remove_file(self.get_save_file_path(file_path));
            None
        }
    }

    /// Delete save file for a given input file
    pub fn delete_save(&self, file_path: &Path) -> io::Result<()> {
        let save_file_path = self.get_save_file_path(file_path);
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
