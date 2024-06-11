mod history;

use history::History;

/// The state of a trial.
pub struct Trial {
    /// The history of typed grapheme clusters and reference grapheme clusters.
    history: History,

    /// The current grapheme cluster being typed.
    /// Once the cluster is finished, it is pushed to `history`.
    working_grapheme_cluster: String,
}

/// A user input to a trial.
pub enum Input {
    /// Type a Unicode scalar value.
    TypeScalar(char),
    /// Delete the last typed grapheme cluster.
    DeleteGraphemeCluster,
    /// Delete the "last word".
    DeleteWord,
}

impl Trial {
    /// Create a new trial.
    pub fn new(env: &crate::Env) -> Self {
        Self {
            history: History::new(env.config.max_misalignment),
            working_grapheme_cluster: String::new(),
        }
    }

    /// Process a user input.
    pub fn process(&mut self, input: Input) {
        match input {
            Input::TypeScalar(c) => {
                self.working_grapheme_cluster.push(c);
            }
            Input::DeleteGraphemeCluster if self.working_grapheme_cluster.is_empty() => {
                todo!()
            }
            Input::DeleteGraphemeCluster => {
                self.working_grapheme_cluster.clear();
            }
            Input::DeleteWord => {
                todo!()
            }
        }
    }
}
