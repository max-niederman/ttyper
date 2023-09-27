use std::{iter, ops::Range};

use unicode_segmentation::UnicodeSegmentation;

/// Stores the history of typed grapheme clusters,
/// and evaluates mistakes using a modified version
/// of the Needleman-Wunsch algorithm for sequence alignment.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    /// The maximum misalignment allowed between the typed and reference strings.
    ///
    /// By only allowing some misalignment, we get O(p min{n, m}) space complexity
    /// and O(p) time complexity for each push, where p is the max misalignment.
    max_misalignment: usize,

    /// The m typed grapheme clusters.
    typed: String,

    /// The starting indices of the typed grapheme clusters.
    typed_indices: Vec<usize>,

    /// The n reference grapheme clusters.
    reference: String,

    /// The starting indices of the reference grapheme clusters.
    reference_indices: Vec<usize>,

    /// Tracked entries in the NW matrix.
    ///
    /// This is a flattened array of m rows, with each containing
    /// the 2p + 1 tracked entries in the corresponding row of the NW matrix.
    /// Note that the first p rows have entries outside the matrix,
    /// which are set to [`NWEntry::Invalid`].
    nw_entries: Vec<u32>,
}

/// An "action" in the Needleman-Wunsch algorithm.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NWAction {
    /// The typed and reference grapheme clusters match.
    Match,

    /// The typed and reference grapheme clusters do not match.
    Mismatch,

    /// The typed grapheme cluster was inserted.
    /// I.e., the reference grapheme cluster was deleted.
    Insertion,

    /// The typed grapheme cluster was deleted.
    /// I.e., the reference grapheme cluster was inserted.
    Deletion,

    /// The entry is invalid, because it is outside the matrix.
    Invalid,
}

impl History {
    /// Creates a new history, with empty typed and reference strings.
    pub fn new(max_misalignment: usize) -> Self {
        let mut nw_entries = vec![u32::MAX; 2 * max_misalignment + 1];

        // the 1,1 entry is always 0
        nw_entries[max_misalignment] = 0;

        Self {
            max_misalignment,
            typed: String::new(),
            typed_indices: Vec::new(),
            reference: String::new(),
            reference_indices: Vec::new(),
            nw_entries,
        }
    }

    /// Pushes a reference grapheme cluster to the history.
    pub fn push_reference(&mut self, reference: &str) {
        debug_assert_eq!(reference.graphemes(true).count(), 1);

        // Push the index of the cluster.
        self.reference_indices.push(self.reference.len());

        // Push to the string.
        self.reference.push_str(reference);

        // The index of the new column.
        let col = self.reference_indices.len();

        // The index of the row being updated.
        let mut row = col.saturating_sub(self.max_misalignment);

        // None of the entries in the row are tracked.
        if row > self.typed_indices.len() {
            return;
        }

        // The index in the tracked entry array of the current entry.
        let mut idx = {
            let row_start = row * self.tracked_entries_per_row();
            let offset = col + self.max_misalignment - row;
            row_start + offset
        };
        // The value of the top neighbor.
        // I.e., the last entry entered.
        let mut top = u32::MAX;
        // The value of the top-left neighbor.
        // I.e., the last entry's left neighbor.
        let mut top_left = idx
            .checked_sub(self.tracked_entries_per_row())
            .map(|i| self.nw_entries[i])
            .unwrap_or(u32::MAX);

        loop {
            let replacement_cost =
                top_left.saturating_add(self.replacement_cost(row, col).unwrap_or(u32::MAX));
            let deletion_cost = top.saturating_add(Self::DELETION_COST);

            // This is the last tracked entry.
            if col + self.max_misalignment == row {
                self.nw_entries[idx] = *[replacement_cost, deletion_cost].iter().min().unwrap();
                break;
            } else {
                let left = self.nw_entries[idx - 1];
                let insertion_cost = left.saturating_add(Self::INSERTION_COST);

                let val = *[deletion_cost, replacement_cost, insertion_cost]
                    .iter()
                    .min()
                    .unwrap();
                self.nw_entries[idx] = val;

                row += 1;
                idx += self.tracked_entries_per_row() - 1;

                if idx >= self.nw_entries.len() {
                    break;
                }

                top = val;
                top_left = left;
            }
        }
    }

    /// Push a typed grapheme cluster to the history.
    fn push_typed(&mut self, typed: &str) {
        debug_assert_eq!(typed.graphemes(true).count(), 1);

        // Push the index of the cluster.
        self.typed_indices.push(self.typed.len());

        // Push to the string.
        self.typed.push_str(typed);

        // The index of the new row.
        let row = self.typed_indices.len();

        // None of the row's entries are tracked.
        if row > self.reference_indices.len() + self.max_misalignment {
            return;
        }

        // The current column.
        // Very high values are outside the reference string.
        let mut col = row.wrapping_sub(self.max_misalignment);

        // A peekable iterator over indices of the previous row.
        // Used to get the top and top-left neighbors.
        let mut prev_row_indices = self.row_indices(row - 1).peekable();
        // The value of the left neighbor.
        let mut left = u32::MAX;

        loop {
            let top_left = match prev_row_indices.next() {
                Some(i) => self.nw_entries[i],
                None => break,
            };

            let top = prev_row_indices
                .peek()
                .map(|&i| self.nw_entries[i])
                .unwrap_or(u32::MAX);

            let replacement_cost =
                top_left.saturating_add(self.replacement_cost(row, col).unwrap_or(u32::MAX));
            let insertion_cost = top.saturating_add(Self::DELETION_COST);
            let deletion_cost = left.saturating_add(Self::INSERTION_COST);

            let val = *[replacement_cost, insertion_cost, deletion_cost]
                .iter()
                .min()
                .unwrap();

            self.nw_entries.push(val);

            col = col.wrapping_add(1);

            left = val;
        }
    }

    // A match is free.
    const MATCH_COST: u32 = 0;
    // A mismatch requires two keystrokes,
    // one to delete the typed grapheme cluster
    // and one to insert the reference grapheme cluster.
    const MISMATCH_COST: u32 = 2;
    // An insertion or deletion requires one keystroke.
    const INSERTION_COST: u32 = 1;
    const DELETION_COST: u32 = 1;

    fn tracked_entries_per_row(&self) -> usize {
        2 * self.max_misalignment + 1
    }

    fn row_indices(&self, row: usize) -> Range<usize> {
        let row_start = row * self.tracked_entries_per_row();
        row_start..row_start + self.tracked_entries_per_row()
    }

    fn col_indices(&self, col: usize) -> impl Iterator<Item = usize> {
        let tracked_entries_per_row = self.tracked_entries_per_row();
        let nw_entries_len = self.nw_entries.len();

        let first =
            (col.saturating_sub(self.max_misalignment) + 1) * self.tracked_entries_per_row() - 1;

        iter::successors(Some(first), move |&i| {
            i.checked_add(tracked_entries_per_row - 1)
        })
        .take_while(move |&i| i < nw_entries_len)
        .take(tracked_entries_per_row)
    }

    fn replacement_cost(&self, row: usize, col: usize) -> Option<u32> {
        let typed_bidx = *self.typed_indices.get(row.checked_sub(1)?)?;
        let reference_bidx = *self.reference_indices.get(col.checked_sub(1)?)?;

        let typed = first_grapheme_cluster(&self.typed[typed_bidx..]);
        let reference = first_grapheme_cluster(&self.reference[reference_bidx..]);

        if typed == reference {
            Some(Self::MATCH_COST)
        } else {
            Some(Self::MISMATCH_COST)
        }
    }
}

fn first_grapheme_cluster(string: &str) -> &str {
    string.graphemes(true).next().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let history = History::new(1);

        assert_eq!(history.max_misalignment, 1);
        assert_eq!(history.typed, "");
        assert_eq!(history.typed_indices, vec![]);
        assert_eq!(history.reference, "");
        assert_eq!(history.reference_indices, vec![]);
        assert_eq!(history.nw_entries[1], 0);
    }

    #[test]
    fn push_saturday_sunday() {
        let mut history = History::new(2);

        let nw_mat = [
            [0, 1, 2, 3, 4, 5, 6, 7, 8],
            [1, 0, 1, 2, 3, 4, 5, 6, 7],
            [2, 1, 2, 3, 2, 3, 4, 5, 6],
            [3, 2, 3, 4, 3, 4, 5, 6, 7],
            [4, 3, 4, 5, 4, 5, 4, 5, 6],
            [5, 4, 3, 4, 5, 6, 5, 4, 5],
            [6, 5, 4, 5, 6, 7, 6, 5, 4],
        ];

        history.push_reference("s");
        check_tracked_rows(&history, &nw_mat);

        history.push_reference("a");
        check_tracked_rows(&history, &nw_mat);

        history.push_reference("t");
        check_tracked_rows(&history, &nw_mat);

        history.push_typed("s");
        check_tracked_rows(&history, &nw_mat);

        history.push_typed("u");
        check_tracked_rows(&history, &nw_mat);
    }

    fn check_tracked_rows<const M: usize, const N: usize>(
        history: &History,
        reference: &[[u32; N]; M],
    ) {
        let rows = history.typed_indices.len() + 1;
        let cols = history.reference_indices.len() + 1;

        let mut tracked = vec![u32::MAX; rows * history.tracked_entries_per_row()];

        for row in 0..rows {
            for col in 0..cols {
                let offset = match (col + history.max_misalignment).checked_sub(row) {
                    Some(offset) if offset < history.tracked_entries_per_row() => offset,
                    _ => continue,
                };
                let tracked_idx = row * history.tracked_entries_per_row() + offset;

                tracked[tracked_idx] = reference[row][col];
            }
        }

        assert_eq!(history.nw_entries, tracked);
    }
}
