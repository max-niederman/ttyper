mod history;

use history::History;

use unicode_segmentation::UnicodeSegmentation;

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
                if let Some(finished_cluster) = push_to_working_cluster(&mut self.working_grapheme_cluster, c) {
                    self.history.push_typed(&finished_cluster);
                }
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

/// Push a scalar value to the working grapheme cluster, returning the previous cluster if it was finished.
fn push_to_working_cluster(working_grapheme_cluster: &mut String, scalar: char) -> Option<String> {
    working_grapheme_cluster.push(scalar);

    let mut clusters = working_grapheme_cluster.graphemes(true);
    let first_cluster = clusters.next().unwrap();

    if let Some(new_cluster) = clusters.next() {
        let first_cluster = first_cluster.to_owned();
        let new_cluster = new_cluster.to_owned();

        *working_grapheme_cluster = new_cluster;

        Some(first_cluster)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_to_working_cluster_handles_latin() {
        let mut working_grapheme_cluster = String::new();

        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'a'),
            None
        );
        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'a'),
            Some("a".to_owned())
        );

        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'é'),
            Some("a".to_owned())
        );
        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'a'),
            Some("é".to_owned())
        );

        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'a'),
            Some("a".to_owned())
        );
        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, '\u{0302}'), // combining circumflex accent
            None
        );
        assert_eq!(
            push_to_working_cluster(&mut working_grapheme_cluster, 'a'),
            Some("a\u{0302}".to_owned())
        );
    }
}
