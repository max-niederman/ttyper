use std::{convert::TryFrom, iter};

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

        // A peekable iterator over indices of the previous column.
        // Used to get the left and top-left neighbors.
        let mut prev_col_indices = self.col_indices(col - 1).peekable();

        // The value of the top neighbor.
        // I.e., the last entry entered.
        let mut top = u32::MAX;

        for EntryIndices { row, buf: idx, .. } in self.col_indices(col) {
            let top_left = if row <= 0 {
                u32::MAX
            } else {
                self.nw_entries[prev_col_indices.next().unwrap().buf]
            };
            let left = prev_col_indices
                .peek()
                .map(|&i| self.nw_entries[i.buf])
                .unwrap_or(u32::MAX);

            let replacement_cost = top_left
                .saturating_add(self.replacement_cost(row, col as isize).unwrap_or(u32::MAX));
            let insertion_cost = top.saturating_add(Self::INSERTION_COST);
            let deletion_cost = left.saturating_add(Self::DELETION_COST);

            let val = *[replacement_cost, insertion_cost, deletion_cost]
                .iter()
                .min()
                .unwrap();

            self.nw_entries[idx] = val;

            top = val;
        }
    }

    /// Push a typed grapheme cluster to the history.
    pub fn push_typed(&mut self, typed: &str) {
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

        // A peekable iterator over indices of the previous row.
        // Used to get the top and top-left neighbors.
        let mut prev_row_indices = self.row_indices(row - 1).peekable();
        // The value of the left neighbor.
        let mut left = u32::MAX;

        self.nw_entries.resize(
            self.nw_entries.len() + self.tracked_entries_per_row(),
            u32::MAX,
        );

        for EntryIndices { col, buf: idx, .. } in self.row_indices(row).filter({
            let cols = self.reference_indices.len() + 1;
            move |&i| i.col < cols as isize
        }) {
            let top_left = match prev_row_indices.next() {
                Some(i) => self.nw_entries[i.buf],
                None => break,
            };

            let top = prev_row_indices
                .peek()
                .map(|&i| self.nw_entries[i.buf])
                .unwrap_or(u32::MAX);

            let replacement_cost = top_left
                .saturating_add(self.replacement_cost(row as isize, col).unwrap_or(u32::MAX));
            let insertion_cost = top.saturating_add(Self::DELETION_COST);
            let deletion_cost = left.saturating_add(Self::INSERTION_COST);

            let val = *[replacement_cost, insertion_cost, deletion_cost]
                .iter()
                .min()
                .unwrap();

            self.nw_entries[idx] = val;

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

    fn row_indices(&self, row: usize) -> impl Iterator<Item = EntryIndices> {
        let first_col = row as isize - self.max_misalignment as isize;

        let first_buf = row * self.tracked_entries_per_row();
        let buf_range = first_buf..first_buf + self.tracked_entries_per_row();

        buf_range
            .zip(first_col..)
            .map(move |(buf, col)| EntryIndices {
                row: row as isize,
                col,
                buf,
            })
    }

    fn col_indices(&self, col: usize) -> impl Iterator<Item = EntryIndices> {
        let tracked_entries_per_row = self.tracked_entries_per_row();
        let nw_entries_len = self.nw_entries.len();

        let first_row = col.saturating_sub(self.max_misalignment) as isize;
        let buf = if first_row == 0 {
            self.max_misalignment + col as usize
        } else {
            (first_row as usize + 1) * tracked_entries_per_row - 1
        };

        let first = EntryIndices {
            row: first_row,
            col: col as isize,
            buf,
        };

        iter::successors(Some(first), move |&EntryIndices { row, col, buf }| {
            Some(EntryIndices {
                row: row + 1,
                col,
                buf: buf + tracked_entries_per_row - 1,
            })
        })
        .take_while(move |&idx| idx.buf < nw_entries_len)
        .take(tracked_entries_per_row)
    }

    fn replacement_cost(&self, row: isize, col: isize) -> Option<u32> {
        let typed_bidx = *self.typed_indices.get(usize::try_from(row - 1).ok()?)?;
        let reference_bidx = *self.reference_indices.get(usize::try_from(col - 1).ok()?)?;

        let typed = first_grapheme_cluster(&self.typed[typed_bidx..]);
        let reference = first_grapheme_cluster(&self.reference[reference_bidx..]);

        if typed == reference {
            Some(Self::MATCH_COST)
        } else {
            Some(Self::MISMATCH_COST)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct EntryIndices {
    row: isize,
    col: isize,
    buf: usize,
}

fn first_grapheme_cluster(string: &str) -> &str {
    string.graphemes(true).next().unwrap()
}

#[cfg(test)]
mod tests {
    use rand::{seq::SliceRandom, SeedableRng};
    use rand_chacha::ChaCha20Rng;

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
    fn reproduces_needleman_wunsch_with_high_max_misalignment() {
        const NW_MAT: [[u32; 9]; 7] = [
            [0, 1, 2, 3, 4, 5, 6, 7, 8],
            [1, 0, 1, 2, 3, 4, 5, 6, 7],
            [2, 1, 2, 3, 2, 3, 4, 5, 6],
            [3, 2, 3, 4, 3, 4, 5, 6, 7],
            [4, 3, 4, 5, 4, 5, 4, 5, 6],
            [5, 4, 3, 4, 5, 6, 5, 4, 5],
            [6, 5, 4, 5, 6, 7, 6, 5, 4],
        ];
        const REFERENCE: &'static str = "saturday";
        const TYPED: &'static str = "sunday";
        const PERMUTATIONS: usize = 1_000;

        let mut rng = ChaCha20Rng::seed_from_u64(42);

        let mut push_order = [vec![true; 6], vec![false; 8]].concat();

        for _ in 0..PERMUTATIONS {
            push_order.shuffle(&mut rng);

            let mut history = History::new(8);

            let mut typed = TYPED.graphemes(true);
            let mut reference = REFERENCE.graphemes(true);

            for &push_typed in &push_order {
                if push_typed {
                    history.push_typed(typed.next().unwrap());
                } else {
                    history.push_reference(reference.next().unwrap());
                }

                check_tracked_rows(&history, &NW_MAT);
            }
        }
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
