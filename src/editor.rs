//! The editor: the document, plus the cursor, viewport, status message, and search.
//!
//! Editing here means "translate a keystroke into a document edit and move the cursor".
//! The document owns the text; the editor owns the cursor, what is visible, the
//! transient status message shown in the message bar, and the current search position.

use std::io;
use std::time::{Duration, Instant};

use crate::document::Document;

/// How long a status message stays on screen.
const MESSAGE_TTL: Duration = Duration::from_secs(5);

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

/// How [`Editor::find`] should step through matches.
#[derive(Clone, Copy)]
pub enum Find {
    /// The query changed: start again from the first row.
    FromStart,
    /// Move to the next match.
    Next,
    /// Move to the previous match.
    Prev,
}

/// A transient status message and when it was set.
struct Message {
    text: String,
    shown_at: Instant,
}

/// A snapshot of the cursor and viewport; a cancelled search restores it.
#[derive(Clone, Copy)]
pub struct ViewState {
    cx: usize,
    cy: usize,
    coloff: usize,
    rowoff: usize,
}

/// Editor state: the document, the cursor, the viewport, the status message, and search.
pub struct Editor {
    document: Document,
    message: Option<Message>,
    /// Row of the current search match, so `Next` / `Prev` can step from it.
    last_match: Option<usize>,
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
            message: None,
            last_match: None,
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

    /// Sets the status message shown in the message bar.
    pub fn set_message(&mut self, text: impl Into<String>) {
        self.message = Some(Message {
            text: text.into(),
            shown_at: Instant::now(),
        });
    }

    /// Clears the status message.
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// The status message, if it was set within the last [`MESSAGE_TTL`].
    #[must_use]
    pub fn message(&self) -> Option<&str> {
        self.message
            .as_ref()
            .filter(|m| m.shown_at.elapsed() < MESSAGE_TTL)
            .map(|m| m.text.as_str())
    }

    /// Sets the file name (used by "Save as").
    pub fn set_filename(&mut self, filename: String) {
        self.document.set_filename(filename);
    }

    /// Captures the cursor and viewport; a cancelled search restores them.
    #[must_use]
    pub fn view_state(&self) -> ViewState {
        ViewState {
            cx: self.cx,
            cy: self.cy,
            coloff: self.coloff,
            rowoff: self.rowoff,
        }
    }

    /// Restores a snapshot taken by [`Editor::view_state`].
    pub fn restore_view(&mut self, view: ViewState) {
        self.cx = view.cx;
        self.cy = view.cy;
        self.coloff = view.coloff;
        self.rowoff = view.rowoff;
    }

    /// Removes the current search highlight.
    pub fn clear_matches(&mut self) {
        self.document.clear_matches();
    }

    /// Searches for `query`, moving the cursor to the match and wrapping around.
    pub fn find(&mut self, query: &str, how: Find) {
        self.document.clear_matches();
        if query.is_empty() {
            self.last_match = None;
            return;
        }

        let count = self.document.len();
        if count == 0 {
            self.last_match = None;
            return;
        }

        let start = match how {
            Find::FromStart => 0,
            Find::Next => self.last_match.map_or(0, |row| (row + 1) % count),
            Find::Prev => self
                .last_match
                .map_or(count - 1, |row| (row + count - 1) % count),
        };

        for offset in 0..count {
            let row = match how {
                Find::Prev => (start + count - offset) % count,
                Find::FromStart | Find::Next => (start + offset) % count,
            };
            if let Some(col) = self.document.row(row).and_then(|line| line.find(query, 0)) {
                self.last_match = Some(row);
                self.cx = col;
                self.cy = row;
                self.document
                    .set_match(row, col..col + query.chars().count());
                return;
            }
        }
        self.last_match = None;
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

    /// Saves the document to its file; returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns the underlying I/O error if the file cannot be written.
    pub fn save(&mut self) -> io::Result<usize> {
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

    #[test]
    fn message_is_readable_until_cleared() {
        let mut e = editor();
        assert_eq!(e.message(), None);
        e.set_message("saved");
        assert_eq!(e.message(), Some("saved"));
        e.clear_message();
        assert_eq!(e.message(), None);
    }

    #[test]
    fn find_moves_to_the_match_and_wraps() {
        let lines = [
            String::from("one"),
            String::from("two one"),
            String::from("three"),
        ];
        let mut e = Editor::new(&lines, None);

        e.find("one", Find::FromStart);
        assert_eq!(e.cursor(), (0, 0));
        e.find("one", Find::Next);
        assert_eq!(e.cursor(), (4, 1));
        e.find("one", Find::Next); // wraps back to the first row
        assert_eq!(e.cursor(), (0, 0));
        e.find("one", Find::Prev); // and backwards to the last match
        assert_eq!(e.cursor(), (4, 1));
    }

    #[test]
    fn find_with_no_match_does_not_move() {
        let mut e = editor();
        e.move_cursor(Move::Down);
        e.find("zzz", Find::FromStart);
        assert_eq!(e.cursor(), (0, 1));
    }

    #[test]
    fn restore_view_puts_the_cursor_back() {
        let mut e = editor();
        let saved = e.view_state();
        e.find("world", Find::FromStart);
        assert_eq!(e.cursor(), (0, 1));
        e.restore_view(saved);
        assert_eq!(e.cursor(), (0, 0));
    }
}
