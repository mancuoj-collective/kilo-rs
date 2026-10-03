//! One line of text, and the conversion between char position and display column.
//!
//! The key fact: **a char does not always occupy one column**.
//! - a tab `\t` occupies 1 to 8 columns (up to the next tab stop);
//! - CJK / fullwidth chars occupy 2 columns;
//! - combining marks occupy 0 columns.
//!
//! The cursor is described by char index (cx) but the screen is drawn by column (rx).
//! This module converts between the two, and slices text by display column.

use std::ops::Range;

use unicode_width::UnicodeWidthChar;

/// Tab stop width.
const TAB_STOP: usize = 8;

/// A run of rendered text, plus whether it belongs to the current search match.
pub struct Segment {
    /// Text to print (tabs expanded; wide chars padded at the window edges).
    pub text: String,
    /// Whether this run is part of the current search match.
    pub matched: bool,
}

/// A single line of text.
pub struct Row {
    chars: Vec<char>,
    /// Char range of the current search match, if any.
    match_range: Option<Range<usize>>,
}

impl Row {
    /// Builds a row from a string.
    #[must_use]
    pub fn new(text: &str) -> Self {
        Self {
            chars: text.chars().collect(),
            match_range: None,
        }
    }

    /// Whether this row is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// Number of chars in this row (not columns).
    #[must_use]
    pub fn len(&self) -> usize {
        self.chars.len()
    }

    /// Total display width of this row, in columns.
    #[must_use]
    pub fn width(&self) -> usize {
        self.chars.iter().fold(0, |col, &ch| advance(col, ch))
    }

    /// Display column of the `cx`-th char.
    #[must_use]
    pub fn cx_to_rx(&self, cx: usize) -> usize {
        self.chars
            .iter()
            .take(cx)
            .fold(0, |col, &ch| advance(col, ch))
    }

    /// Index of the char at display column `rx`.
    #[must_use]
    pub fn rx_to_cx(&self, rx: usize) -> usize {
        let mut col = 0;
        for (index, &ch) in self.chars.iter().enumerate() {
            let next = advance(col, ch);
            if next > rx {
                return index;
            }
            col = next;
        }
        self.len()
    }

    /// Index of the first occurrence of `query` at or after char index `from`.
    #[must_use]
    pub fn find(&self, query: &str, from: usize) -> Option<usize> {
        let needle: Vec<char> = query.chars().collect();
        if needle.is_empty() || from > self.chars.len() {
            return None;
        }

        let haystack = &self.chars[from..];
        if needle.len() > haystack.len() {
            return None;
        }

        (0..=haystack.len() - needle.len())
            .find(|&i| haystack[i..i + needle.len()] == needle[..])
            .map(|i| from + i)
    }

    /// Marks `range` (char indices) as the current search match; `None` clears it.
    pub fn set_match(&mut self, range: Option<Range<usize>>) {
        self.match_range = range;
    }

    /// Returns display columns `[coloff, coloff + width)`, split into highlighted and plain runs.
    #[must_use]
    pub fn render_window(&self, coloff: usize, width: usize) -> Vec<Segment> {
        let mut segments = Vec::new();
        let end = coloff + width;
        let mut col = 0;

        for (index, &ch) in self.chars.iter().enumerate() {
            let start = col;
            col = advance(col, ch);

            if col <= coloff {
                continue; // entirely left of the window
            }
            if start >= end {
                break; // window is full
            }

            let matched = self
                .match_range
                .as_ref()
                .is_some_and(|range| range.contains(&index));

            if ch == '\t' || start < coloff || col > end {
                // Expand tabs to spaces; pad a wide char straddling the edge with spaces.
                let from = start.max(coloff);
                let to = col.min(end);
                push_segment(&mut segments, &" ".repeat(to - from), matched);
            } else {
                let mut buf = [0u8; 4];
                push_segment(&mut segments, ch.encode_utf8(&mut buf), matched);
            }
        }

        segments
    }

    /// The row's characters as a string (used when saving).
    #[must_use]
    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    /// Inserts `ch` at char index `cx` (clamped to the row length).
    pub fn insert_char(&mut self, cx: usize, ch: char) {
        let at = cx.min(self.len());
        self.chars.insert(at, ch);
    }

    /// Removes the char at `cx`, if it exists.
    pub fn delete_char(&mut self, cx: usize) {
        if cx < self.len() {
            self.chars.remove(cx);
        }
    }

    /// Splits the row at `cx`, returning the tail as a new row.
    #[must_use]
    pub fn split_off(&mut self, cx: usize) -> Self {
        let at = cx.min(self.len());
        Self {
            chars: self.chars.split_off(at),
            match_range: None,
        }
    }

    /// Appends another row's characters to the end of this one.
    pub fn append(&mut self, other: &mut Row) {
        self.chars.append(&mut other.chars);
    }
}

/// Appends `text` to `segments`, merging it into the previous run when the match flag agrees.
fn push_segment(segments: &mut Vec<Segment>, text: &str, matched: bool) {
    if text.is_empty() {
        return;
    }
    match segments.last_mut() {
        Some(last) if last.matched == matched => last.text.push_str(text),
        _ => segments.push(Segment {
            text: text.to_owned(),
            matched,
        }),
    }
}

/// Display width of `ch` starting at column `col` (tabs depend on `col`).
fn advance(col: usize, ch: char) -> usize {
    if ch == '\t' {
        col + (TAB_STOP - col % TAB_STOP)
    } else {
        col + ch.width().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Joins the segments back into plain text.
    fn render_text(row: &Row, coloff: usize, width: usize) -> String {
        row.render_window(coloff, width)
            .iter()
            .map(|segment| segment.text.as_str())
            .collect()
    }

    #[test]
    fn wide_chars_take_two_columns() {
        let row = Row::new("中a");
        assert_eq!(row.cx_to_rx(0), 0);
        assert_eq!(row.cx_to_rx(1), 2);
        assert_eq!(row.cx_to_rx(2), 3);
        assert_eq!(row.width(), 3);
    }

    #[test]
    fn tab_aligns_to_next_stop() {
        let row = Row::new("\tx");
        assert_eq!(row.cx_to_rx(1), 8);
        assert_eq!(row.cx_to_rx(2), 9);
    }

    #[test]
    fn rx_to_cx_is_the_inverse() {
        let row = Row::new("中a");
        assert_eq!(row.rx_to_cx(1), 0);
        assert_eq!(row.rx_to_cx(2), 1);
    }

    #[test]
    fn render_window_clips_by_columns() {
        let row = Row::new("中文");
        assert_eq!(render_text(&row, 0, 4), "中文");
        assert_eq!(render_text(&row, 0, 3), "中 ");
    }

    #[test]
    fn render_window_marks_the_match() {
        let mut row = Row::new("hello world");
        row.set_match(Some(6..11));

        let segments = row.render_window(0, 11);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "hello ");
        assert!(!segments[0].matched);
        assert_eq!(segments[1].text, "world");
        assert!(segments[1].matched);
    }

    #[test]
    fn find_reports_char_index_and_can_skip() {
        let row = Row::new("abcabc");
        assert_eq!(row.find("bc", 0), Some(1));
        assert_eq!(row.find("bc", 2), Some(4));
        assert_eq!(row.find("zz", 0), None);
    }

    #[test]
    fn split_and_append_round_trip() {
        let mut row = Row::new("abcd");
        let mut tail = row.split_off(2);
        assert_eq!(row.text(), "ab");
        assert_eq!(tail.text(), "cd");
        row.append(&mut tail);
        assert_eq!(row.text(), "abcd");
    }
}
