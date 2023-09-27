mod history;

use history::History;

/// The state of a trial.
pub struct Trial {
    /// The history of typed grapheme clusters and reference grapheme clusters.
    history: History,

    /// The current grapheme cluster being typed.
    /// Once the cluster is finished, it is pushed to `evaluation`.
    working_grapheme_cluster: String,
}


