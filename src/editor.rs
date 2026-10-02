//! Editable editor state: cursor, viewport, and rendering by display column.

use crate::row::Row;

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

/// Editor state: buffer, cursor, and viewport.
pub struct Editor {
    rows: Vec<Row>,
    /// Cursor position in **chars**: `cx` is the char index, `cy` the row.
    cx: usize,
    cy: usize,
    /// Top-left of the viewport, in **display columns / rows**.
    coloff: usize,
    rowoff: usize,
}

impl Editor {
    /// Creates a new editor from the given lines.
    #[must_use]
    pub fn new(lines: &[String]) -> Self {
        Self {
            rows: lines.iter().map(|line| Row::new(line)).collect(),
            cx: 0,
            cy: 0,
            coloff: 0,
            rowoff: 0,
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

    /// Renders row `index` at display columns `[coloff, coloff + width)`.
    #[must_use]
    pub fn render_line(&self, index: usize, coloff: usize, width: usize) -> Option<String> {
        self.rows
            .get(index)
            .map(|row| row.render_window(coloff, width))
    }

    /// Moves the cursor in the given direction (in chars).
    pub fn move_cursor(&mut self, direction: Move) {
        let numrows = self.rows.len();
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
        self.rows.get(self.cy).map_or(0, Row::len)
    }

    /// Display column of the cursor.
    fn current_rx(&self) -> usize {
        self.rows
            .get(self.cy)
            .map_or(0, |row| row.cx_to_rx(self.cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn editor() -> Editor {
        Editor::new(&[String::from("hello"), String::from("world!!")])
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
        let mut e = Editor::new(&lines);
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down);
        e.move_cursor(Move::Down);
        e.scroll(3, 20);
        assert_eq!(e.viewport(), (0, 1));
    }

    #[test]
    fn cursor_reports_display_columns_for_wide_chars() {
        let mut e = Editor::new(&[String::from("中文")]);
        e.move_cursor(Move::Right);
        assert_eq!(e.cursor(), (2, 0));
        e.move_cursor(Move::End);
        assert_eq!(e.cursor(), (4, 0));
    }
}
