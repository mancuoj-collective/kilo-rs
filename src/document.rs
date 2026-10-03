//! The text buffer: rows, file name, dirty flag, syntax choice, and line-level edits.
//!
//! This is the domain model. It knows nothing about the cursor or the terminal; it only
//! answers "what is on row N" and "change row N at char C". It also keeps each row's
//! syntax highlighting up to date, since that depends only on the text.

use std::io;
use std::ops::Range;

use crate::{row::Row, syntax::Syntax};

/// The edited document: a list of rows plus the file metadata.
pub struct Document {
    rows: Vec<Row>,
    filename: Option<String>,
    dirty: bool,
    syntax: Option<&'static Syntax>,
}

impl Document {
    /// Builds a document from lines and an optional file name.
    #[must_use]
    pub fn new(lines: &[String], filename: Option<String>) -> Self {
        let mut document = Self {
            rows: lines.iter().map(|line| Row::new(line)).collect(),
            filename,
            dirty: false,
            syntax: None,
        };
        document.select_syntax();
        document
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

    /// Filetype name for the status bar (e.g. `"c"`, or `"no ft"` when unknown).
    #[must_use]
    pub fn filetype(&self) -> &'static str {
        self.syntax.map_or("no ft", |syntax| syntax.filetype)
    }

    /// Sets the file name (used by "Save as"); re-selects the syntax.
    pub fn set_filename(&mut self, filename: String) {
        self.filename = Some(filename);
        self.select_syntax();
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

    /// Highlights `range` (char indices) as the current search match on `row`.
    pub fn set_match(&mut self, row: usize, range: Range<usize>) {
        if let Some(row) = self.rows.get_mut(row) {
            row.set_match(Some(range));
        }
    }

    /// Removes the search highlight from `row`.
    pub fn clear_match(&mut self, row: usize) {
        if let Some(row) = self.rows.get_mut(row) {
            row.set_match(None);
        }
    }

    /// Removes the search highlight from every row.
    pub fn clear_matches(&mut self) {
        for row in &mut self.rows {
            row.set_match(None);
        }
    }

    /// Highlights all rows from scratch. Call this after the syntax changes.
    fn highlight_all(&mut self) {
        let Some(syntax) = self.syntax else {
            return;
        };
        let mut depth = 0;
        for row in &mut self.rows {
            let (hl, ends) = syntax.highlight(row.chars(), depth);
            row.set_highlight(hl);
            row.set_comment_depth(ends);
            depth = ends;
        }
    }

    /// Re-highlights from row `from`, continuing only while the comment depth changes
    /// (the same short-circuit kilo uses for its open-comment flag).
    fn refresh_syntax(&mut self, from: usize) {
        let Some(syntax) = self.syntax else {
            return;
        };
        let mut depth = if from > 0 {
            self.rows[from - 1].comment_depth()
        } else {
            0
        };
        for row in &mut self.rows[from..] {
            let (hl, ends) = syntax.highlight(row.chars(), depth);
            let changed = row.comment_depth() != ends;
            row.set_highlight(hl);
            row.set_comment_depth(ends);
            if !changed {
                break;
            }
            depth = ends;
        }
    }

    /// Picks a syntax from the file name and (re-)highlights the whole document.
    fn select_syntax(&mut self) {
        self.syntax = self.filename.as_deref().and_then(Syntax::for_path);
        self.highlight_all();
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
        self.refresh_syntax(cy);
    }

    /// Deletes the char at `cx` within row `cy`.
    pub fn delete_char(&mut self, cy: usize, cx: usize) {
        if let Some(row) = self.rows.get_mut(cy) {
            row.delete_char(cx);
        }
        self.dirty = true;
        self.refresh_syntax(cy);
    }

    /// Inserts an empty row at `at`.
    pub fn insert_empty_row(&mut self, at: usize) {
        self.rows.insert(at, Row::new(""));
        self.dirty = true;
        self.refresh_syntax(at);
    }

    /// Splits row `cy` at `cx`, moving the tail into a new row below it.
    pub fn split_line(&mut self, cy: usize, cx: usize) {
        // End the mutable borrow of `self.rows` before inserting into it.
        let tail = self.rows.get_mut(cy).map(|row| row.split_off(cx));
        if let Some(tail) = tail {
            self.rows.insert(cy + 1, tail);
        }
        self.dirty = true;
        self.refresh_syntax(cy);
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
        self.refresh_syntax(cy - 1);
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

    /// Writes the document to its file; returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if the file cannot be written.
    pub fn save(&mut self) -> io::Result<usize> {
        let Some(path) = &self.filename else {
            return Ok(0);
        };
        let text = self.serialize();
        std::fs::write(path, &text)?;
        self.dirty = false;
        Ok(text.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::Highlight;

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

    #[test]
    fn syntax_is_selected_from_the_file_name() {
        let d = Document::new(&[String::from("int x;")], Some(String::from("a.c")));
        assert_eq!(d.filetype(), "c");
        assert_eq!(d.row(0).unwrap().highlight()[0], Highlight::Keyword2);

        let d = Document::new(&[String::from("hi")], Some(String::from("a.txt")));
        assert_eq!(d.filetype(), "no ft");
    }

    #[test]
    fn multi_line_comment_state_propagates_on_edit() {
        let lines = [String::from("/*"), String::from("x")];
        let mut d = Document::new(&lines, Some(String::from("a.c")));
        assert_eq!(d.row(0).unwrap().comment_depth(), 1);
        assert_eq!(
            d.row(1).unwrap().highlight()[0],
            Highlight::MultilineComment
        );

        // Closing the comment on row 0 clears the state on row 1.
        d.insert_char(0, 2, '*');
        d.insert_char(0, 3, '/');
        assert_eq!(d.row(0).unwrap().comment_depth(), 0);
        assert_eq!(d.row(1).unwrap().highlight()[0], Highlight::Normal);
    }
}
