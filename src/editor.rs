//! The editor: the document, plus the cursor and viewport.
//!
//! Editing here means "translate a keystroke into a document edit and move the cursor".
//! The document owns the text; the editor owns the cursor and what is visible.

use std::io;

use crate::document::Document;

/// A cursor movement direction.
#[derive(Clone, Copy)]
pub enum Move {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
}

/// Editor state: the document, the cursor, and the viewport.
pub struct Editor {
    document: Document,
    /// Cursor position in **chars**: `cx` is the char index, `cy` the row.
    cx: usize,
    cy: usize,
    /// Top-left of the viewport, in **display columns / rows**.
    coloff: usize,
    rowoff: usize,
}

impl Editor {
    /// Creates a new editor from the given lines and optional file name.
    #[must_use]
    pub fn new(lines: &[String], filename: Option<String>) -> Self {
        Self {
            document: Document::new(lines, filename),
            cx: 0,
            cy: 0,
            coloff: 0,
            rowoff: 0,
        }
    }

    /// The document, for rendering.
    #[must_use]
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Cursor position `(display column, row)`; the column is a display column, not a char index.
    #[must_use]
    pub fn cursor(&self) -> (usize, usize) {
        (self.current_rx(), self.cy)
    }

    /// Top-left of the viewport, as `(display column, row)`.
    #[must_use]
    pub fn viewport(&self) -> (usize, usize) {
        (self.coloff, self.rowoff)
    }

    /// Inserts `ch` at the cursor.
    pub fn insert_char(&mut self, ch: char) {
        self.document.insert_char(self.cy, self.cx, ch);
        self.cx += 1;
    }

    /// Splits the current row at the cursor (Enter / Return).
    pub fn insert_newline(&mut self) {
        if self.cx == 0 || self.cy >= self.document.len() {
            self.document.insert_empty_row(self.cy);
        } else {
            self.document.split_line(self.cy, self.cx);
        }
        self.cy += 1;
        self.cx = 0;
    }

    /// Deletes the char before the cursor; at column 0 it joins with the previous row.
    pub fn delete_char(&mut self) {
        if self.cy >= self.document.len() {
            return;
        }
        if self.cx == 0 {
            if self.cy == 0 {
                return;
            }
            self.cx = self.document.join_lines(self.cy);
            self.cy -= 1;
        } else {
            self.document.delete_char(self.cy, self.cx - 1);
            self.cx -= 1;
        }
    }

    /// Saves the document to its file.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if the file cannot be written.
    pub fn save(&mut self) -> io::Result<()> {
        self.document.save()
    }

    /// Moves the cursor in the given direction (in chars).
    pub fn move_cursor(&mut self, direction: Move) {
        let numrows = self.document.len();
        match direction {
            Move::Left => {
                if self.cx > 0 {
                    self.cx -= 1;
                } else if self.cy > 0 {
                    self.cy -= 1;
                    self.cx = self.current_line_len();
                }
            }
            Move::Right => {
                if self.cy < numrows {
                    if self.cx < self.current_line_len() {
                        self.cx += 1;
                    } else {
                        self.cy += 1;
                        self.cx = 0;
                    }
                }
            }
            Move::Up => {
                if self.cy > 0 {
                    self.cy -= 1;
                }
            }
            Move::Down => {
                if self.cy < numrows {
                    self.cy += 1;
                }
            }
            Move::Home => self.cx = 0,
            Move::End => self.cx = self.current_line_len(),
        }

        // Moving to a shorter line: clamp the column to the line end.
        self.cx = self.cx.min(self.current_line_len());
    }

    /// Adjusts the viewport so the cursor stays visible; horizontally by **display column** `rx`.
    pub fn scroll(&mut self, rows: usize, cols: usize) {
        let rx = self.current_rx();

        if self.cy < self.rowoff {
            self.rowoff = self.cy;
        }
        if self.cy >= self.rowoff + rows {
            self.rowoff = self.cy - rows + 1;
        }
        if rx < self.coloff {
            self.coloff = rx;
        }
        if rx >= self.coloff + cols {
            self.coloff = rx - cols + 1;
        }
    }

    /// Number of chars in the cursor's row.
    fn current_line_len(&self) -> usize {
        self.document.line_len(self.cy)
    }

    /// Display column of the cursor.
    fn current_rx(&self) -> usize {
        self.document
            .row(self.cy)
            .map_or(0, |row| row.cx_to_rx(self.cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::new(&[String::from("hello"), String::from("world!!")], None)
    }

    #[test]
    fn right_moves_within_a_line() {
        let mut e = editor();
        e.move_cursor(Move::Right);
        assert_eq!(e.cursor(), (1, 0));
    }

    #[test]
    fn moving_up_clamps_column_to_line_end() {
        let mut e = editor();
        e.move_cursor(Move::End);
        e.move_cursor(Move::Down);
        assert_eq!(e.cursor(), (5, 1));
        e.move_cursor(Move::Up);
        assert_eq!(e.cursor(), (5, 0));
    }

    #[test]
    fn scroll_keeps_cursor_visible() {
        let lines: Vec<String> = (0..10).map(|i| format!("line {i}")).collect();
        let mut e = Editor::new(&lines, None);
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down);
        e.scroll(3, 20);
        assert_eq!(e.viewport(), (0, 1));
    }

    #[test]
    fn cursor_reports_display_columns_for_wide_chars() {
        let mut e = Editor::new(&[String::from("中文")], None);
        e.move_cursor(Move::Right);
        assert_eq!(e.cursor(), (2, 0));
        e.move_cursor(Move::End);
        assert_eq!(e.cursor(), (4, 0));
    }

    #[test]
    fn insert_char_updates_row_and_cursor() {
        let mut e = Editor::new(&[String::from("ab")], None);
        e.move_cursor(Move::End);
        e.insert_char('c');
        assert_eq!(e.cursor(), (3, 0));
        assert_eq!(e.document().serialize(), "abc\n");
        assert!(e.document().is_dirty());
    }

    #[test]
    fn enter_splits_the_line() {
        let mut e = Editor::new(&[String::from("abcd")], None);
        e.move_cursor(Move::Right);
        e.move_cursor(Move::Right); // cx = 2
        e.insert_newline();
        assert_eq!(e.cursor(), (0, 1));
        assert_eq!(e.document().serialize(), "ab\ncd\n");
    }

    #[test]
    fn backspace_at_column_zero_joins_lines() {
        let mut e = editor();
        e.move_cursor(Move::Down); // cy = 1, cx = 0
        e.delete_char();
        assert_eq!(e.cursor(), (5, 0));
        assert_eq!(e.document().serialize(), "helloworld!!\n");
    }
}
