//! One line of text, and the conversion between char position and display column.
//!
//! The key fact: **a char does not always occupy one column**.
//! - a tab `\t` occupies 1 to 8 columns (up to the next tab stop);
//! - CJK / fullwidth chars occupy 2 columns;
//! - combining marks occupy 0 columns.
//!
//! The cursor is described by char index (cx) but the screen is drawn by column (rx).
//! This module converts between the two, and slices text by display column.
//!
//! A row also carries, per character, a [`Highlight`] category (filled in by
//! `syntax`) and whether it ends inside a multi-line comment.

use std::ops::Range;

use unicode_width::UnicodeWidthChar;

use crate::syntax::Highlight;

/// Tab stop width.
const TAB_STOP: usize = 8;

/// A run of rendered text plus its style: syntax category and search-match overlay.
pub struct Segment {
    /// Text to print (tabs expanded; wide chars padded at the window edges).
    pub text: String,
    /// Syntax-highlight category of this run.
    pub highlight: Highlight,
    /// Whether this run is part of the current search match.
    pub matched: bool,
    /// Whether this run is a non-printable char shown as a symbol.
    pub control: bool,
}

/// A single line of text.
pub struct Row {
    chars: Vec<char>,
    /// Syntax category per char (same length as `chars`).
    hl: Vec<Highlight>,
    /// Block-comment depth at the end of this row (0 = not in a comment).
    comment_depth: u32,
    /// Char range of the current search match, if any.
    match_range: Option<Range<usize>>,
}

impl Row {
    /// Builds a row from a string.
    #[must_use]
    pub fn new(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let hl = vec![Highlight::Normal; chars.len()];
        Self {
            chars,
            hl,
            comment_depth: 0,
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

    /// The row's characters (read-only, for the syntax lexer).
    #[must_use]
    pub fn chars(&self) -> &[char] {
        &self.chars
    }

    /// The syntax categories, one per char.
    #[must_use]
    pub fn highlight(&self) -> &[Highlight] {
        &self.hl
    }

    /// Replaces the syntax categories (length must match `chars`).
    pub fn set_highlight(&mut self, hl: Vec<Highlight>) {
        self.hl = hl;
    }

    /// Block-comment depth at the end of this row (0 = not in a comment).
    #[must_use]
    pub fn comment_depth(&self) -> u32 {
        self.comment_depth
    }

    /// Sets the block-comment depth at the end of this row.
    pub fn set_comment_depth(&mut self, depth: u32) {
        self.comment_depth = depth;
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
        if needle.is_empty() || from > self.len() {
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

    /// Index of the last occurrence of `query` before char index `before` (`before` is
    /// exclusive); used to step backwards within a row.
    #[must_use]
    pub fn find_last(&self, query: &str, before: usize) -> Option<usize> {
        let mut found = None;
        let mut from = 0;
        while let Some(at) = self.find(query, from) {
            if at >= before {
                break;
            }
            found = Some(at);
            from = at + 1;
        }
        found
    }

    /// Marks `range` (char indices) as the current search match; `None` clears it.
    pub fn set_match(&mut self, range: Option<Range<usize>>) {
        self.match_range = range;
    }

    /// Returns display columns `[coloff, coloff + width)`, split into styled runs.
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

            let highlight = self.hl.get(index).copied().unwrap_or(Highlight::Normal);
            let matched = self
                .match_range
                .as_ref()
                .is_some_and(|range| range.contains(&index));

            if ch == '\t' || start < coloff || col > end {
                // Expand tabs to spaces; pad a wide char straddling the edge with spaces.
                let from = start.max(coloff);
                let to = col.min(end);
                push_segment(
                    &mut segments,
                    &" ".repeat(to - from),
                    highlight,
                    matched,
                    false,
                );
            } else {
                // Non-printable chars are shown as a symbol, never written to the terminal raw.
                let control = ch.is_control();
                let rendered = if control { control_symbol(ch) } else { ch };
                let mut buf = [0u8; 4];
                push_segment(
                    &mut segments,
                    rendered.encode_utf8(&mut buf),
                    highlight,
                    matched,
                    control,
                );
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
        self.hl.insert(at, Highlight::Normal);
    }

    /// Removes the char at `cx`, if it exists.
    pub fn delete_char(&mut self, cx: usize) {
        if cx < self.len() {
            self.chars.remove(cx);
            self.hl.remove(cx);
        }
    }

    /// Splits the row at `cx`, returning the tail as a new row.
    #[must_use]
    pub fn split_off(&mut self, cx: usize) -> Self {
        let at = cx.min(self.len());
        Self {
            chars: self.chars.split_off(at),
            hl: self.hl.split_off(at),
            comment_depth: 0,
            match_range: None,
        }
    }

    /// Appends another row's characters to the end of this one.
    pub fn append(&mut self, other: &mut Row) {
        self.chars.append(&mut other.chars);
        self.hl.append(&mut other.hl);
    }
}

/// Appends `text` to `segments`, merging it into the previous run when all style keys agree.
fn push_segment(
    segments: &mut Vec<Segment>,
    text: &str,
    highlight: Highlight,
    matched: bool,
    control: bool,
) {
    if text.is_empty() {
        return;
    }
    match segments.last_mut() {
        Some(last)
            if last.highlight == highlight
                && last.matched == matched
                && last.control == control =>
        {
            last.text.push_str(text);
        }
        _ => segments.push(Segment {
            text: text.to_owned(),
            highlight,
            matched,
            control,
        }),
    }
}

/// Display width of `ch` starting at column `col` (tabs depend on `col`).
fn advance(col: usize, ch: char) -> usize {
    if ch == '\t' {
        col + (TAB_STOP - col % TAB_STOP)
    } else {
        col + display_width(ch)
    }
}

/// Columns a char occupies; a non-printable char is shown as a one-column symbol.
fn display_width(ch: char) -> usize {
    if ch.is_control() {
        1
    } else {
        ch.width().unwrap_or(0)
    }
}

/// The symbol shown for a non-printable char (`\x01` → `A`, others → `?`), like kilo.
fn control_symbol(ch: char) -> char {
    match u32::from(ch) {
        code @ 0..=26 => char::from(b'@' + u8::try_from(code).expect("code fits in a byte")),
        _ => '?',
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
    fn render_window_carries_the_highlight() {
        let mut row = Row::new("int x");
        row.set_highlight(vec![
            Highlight::Keyword2,
            Highlight::Keyword2,
            Highlight::Keyword2,
            Highlight::Normal,
            Highlight::Normal,
        ]);

        let segments = row.render_window(0, 5);
        assert_eq!(segments[0].text, "int");
        assert_eq!(segments[0].highlight, Highlight::Keyword2);
        assert_eq!(segments[1].text, " x");
        assert_eq!(segments[1].highlight, Highlight::Normal);
    }

    #[test]
    fn control_chars_are_shown_as_symbols() {
        let row = Row::new("a\x01b");
        assert_eq!(row.width(), 3); // the control char takes one column
        let segments = row.render_window(0, 3);
        assert!(segments.iter().any(|segment| segment.control));
        let text: String = segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect();
        assert_eq!(text, "aAb"); // \x01 renders as 'A'
    }

    #[test]
    fn find_reports_char_index_and_can_skip() {
        let row = Row::new("abcabc");
        assert_eq!(row.find("bc", 0), Some(1));
        assert_eq!(row.find("bc", 2), Some(4));
        assert_eq!(row.find("zz", 0), None);
    }

    #[test]
    fn find_last_reports_the_last_match_before_a_column() {
        let row = Row::new("a.b.a.b");
        assert_eq!(row.find_last("a", row.len()), Some(4));
        assert_eq!(row.find_last("a", 4), Some(0));
        assert_eq!(row.find_last("a", 0), None);
    }

    #[test]
    fn split_and_append_round_trip() {
        let mut row = Row::new("abcd");
        let mut tail = row.split_off(2);
        assert_eq!(row.text(), "ab");
        assert_eq!(tail.text(), "cd");
        assert_eq!(row.highlight().len(), 2);
        assert_eq!(tail.highlight().len(), 2);
        row.append(&mut tail);
        assert_eq!(row.text(), "abcd");
        assert_eq!(row.highlight().len(), 4);
    }
}
