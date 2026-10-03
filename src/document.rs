//! The text buffer: rows, file name, dirty flag, and line-level edits.
//!
//! This is the domain model. It knows nothing about the cursor or the terminal; it only
//! answers "what is on row N" and "change row N at char C".

use std::io;

use crate::row::Row;

/// The edited document: a list of rows plus the file metadata.
pub struct Document {
    rows: Vec<Row>,
    filename: Option<String>,
    dirty: bool,
}

impl Document {
    /// Builds a document from lines and an optional file name.
    #[must_use]
    pub fn new(lines: &[String], filename: Option<String>) -> Self {
        Self {
            rows: lines.iter().map(|line| Row::new(line)).collect(),
            filename,
            dirty: false,
        }
    }

    /// Number of rows.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether there are no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Whether the document has unsaved changes.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// File name, if any.
    #[must_use]
    pub fn filename(&self) -> Option<&str> {
        self.filename.as_deref()
    }

    /// The row at `index`, if it exists.
    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// Number of chars in row `cy`; 0 if the row doesn't exist.
    #[must_use]
    pub fn line_len(&self, cy: usize) -> usize {
        self.rows.get(cy).map_or(0, Row::len)
    }

    /// Inserts `ch` at char `cx` of row `cy`. Creates the row when `cy` is past the end.
    pub fn insert_char(&mut self, cy: usize, cx: usize, ch: char) {
        if cy == self.rows.len() {
            self.rows.push(Row::new(""));
        }
        if let Some(row) = self.rows.get_mut(cy) {
            row.insert_char(cx, ch);
        }
        self.dirty = true;
    }

    /// Deletes the char at `cx` within row `cy`.
    pub fn delete_char(&mut self, cy: usize, cx: usize) {
        if let Some(row) = self.rows.get_mut(cy) {
            row.delete_char(cx);
        }
        self.dirty = true;
    }

    /// Inserts an empty row at `at`.
    pub fn insert_empty_row(&mut self, at: usize) {
        self.rows.insert(at, Row::new(""));
        self.dirty = true;
    }

    /// Splits row `cy` at `cx`, moving the tail into a new row below it.
    pub fn split_line(&mut self, cy: usize, cx: usize) {
        // End the mutable borrow of `self.rows` before inserting into it.
        let tail = self.rows.get_mut(cy).map(|row| row.split_off(cx));
        if let Some(tail) = tail {
            self.rows.insert(cy + 1, tail);
        }
        self.dirty = true;
    }

    /// Merges row `cy` into row `cy - 1`; returns the join column (the previous row's length).
    ///
    /// `cy` must be greater than 0.
    pub fn join_lines(&mut self, cy: usize) -> usize {
        debug_assert!(cy > 0, "join_lines needs a row below the first");
        // Own the current row first, so the previous row can be borrowed mutably.
        let mut row = self.rows.remove(cy);
        let prev = &mut self.rows[cy - 1];
        let col = prev.len();
        prev.append(&mut row);
        self.dirty = true;
        col
    }

    /// The whole document as text, rows joined by `\n`.
    #[must_use]
    pub fn serialize(&self) -> String {
        let mut out = String::new();
        for row in &self.rows {
            out.push_str(&row.text());
            out.push('\n');
        }
        out
    }

    /// Writes the document to its file. Does nothing when there is no file name.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if the file cannot be written.
    pub fn save(&mut self) -> io::Result<()> {
        let Some(path) = &self.filename else {
            return Ok(());
        };
        std::fs::write(path, self.serialize())?;
        self.dirty = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc() -> Document {
        Document::new(&[String::from("ab"), String::from("cd")], None)
    }

    #[test]
    fn split_line_moves_the_tail_down() {
        let mut d = doc();
        d.split_line(0, 1);
        assert_eq!(d.serialize(), "a\nb\ncd\n");
    }

    #[test]
    fn join_lines_merges_upward_and_reports_column() {
        let mut d = doc();
        let col = d.join_lines(1);
        assert_eq!(col, 2);
        assert_eq!(d.serialize(), "abcd\n");
    }

    #[test]
    fn editing_marks_dirty() {
        let mut d = doc();
        assert!(!d.is_dirty());
        d.insert_char(0, 0, 'x');
        assert!(d.is_dirty());
        assert_eq!(d.serialize(), "xab\ncd\n");
    }
}
