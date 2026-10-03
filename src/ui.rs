//! Rendering: turn the editor state into terminal output.
//!
//! This layer is pure with respect to the editor: it only reads it and writes to the
//! terminal. Scrolling the viewport is *not* done here — the caller does that before
//! calling [`draw`], so rendering stays a function of the current state.
//!
//! Mapping a [`Highlight`] category to a color lives here too: `syntax` names the
//! categories, `ui` decides how they look.

use std::io::{self, Write};

use crossterm::{
    cursor::{self, MoveTo},
    queue,
    style::{Attribute, Color, Print, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};

use crate::{editor::Editor, row::Segment, syntax::Highlight};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Draws one frame: the buffer, a status bar, a message bar, and the editor cursor.
///
/// The bottom two rows are reserved for the status bar and the message bar.
///
/// # Errors
///
/// Returns the underlying I/O error if writing to the terminal fails.
///
/// # Panics
///
/// Panics if the cursor is outside the viewport — an invariant that [`Editor::scroll`]
/// maintains, so the caller must run it before drawing.
pub fn draw(editor: &Editor, cols: u16, rows: u16) -> io::Result<()> {
    let width = cols as usize;
    let height = rows as usize;
    let text_height = height.saturating_sub(2); // status bar + message bar

    let (coloff, rowoff) = editor.viewport();
    let (rx, cy) = editor.cursor();
    let document = editor.document();

    let mut stdout = io::stdout();
    queue!(stdout, cursor::Hide, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..text_height {
        let filerow = rowoff + y;
        if let Some(row) = document.row(filerow) {
            for segment in row.render_window(coloff, width) {
                draw_segment(&mut stdout, &segment)?;
            }
        } else if document.is_empty() && y == text_height / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
        }

        if y + 1 < height {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    draw_status_bar(&mut stdout, editor, cols, rows)?;
    draw_message_bar(&mut stdout, editor, cols, rows)?;

    // The viewport guarantees the cursor is on screen, so these conversions can't fail.
    let cursor_col =
        u16::try_from(rx - coloff).expect("cursor column should be inside the viewport");
    let cursor_row = u16::try_from(cy - rowoff).expect("cursor row should be inside the viewport");
    queue!(stdout, MoveTo(cursor_col, cursor_row), cursor::Show)?;
    stdout.flush()
}

/// Writes one rendered segment with its syntax color and/or match overlay.
fn draw_segment(stdout: &mut io::Stdout, segment: &Segment) -> io::Result<()> {
    let color = highlight_color(segment.highlight);
    if let Some(color) = color {
        queue!(stdout, SetForegroundColor(color))?;
    }
    let reverse = segment.matched || segment.control;
    if reverse {
        queue!(stdout, SetAttribute(Attribute::Reverse))?;
    }
    queue!(stdout, Print(&segment.text))?;
    if reverse {
        queue!(stdout, SetAttribute(Attribute::Reset))?;
    }
    if color.is_some() {
        queue!(stdout, SetForegroundColor(Color::Reset))?;
    }
    Ok(())
}

/// The terminal color for a syntax category (`None` = the default foreground).
///
/// These are the standard ANSI slots kilo uses (`editorSyntaxToColor`): cyan comments,
/// yellow keywords, green type keywords, magenta strings, red numbers.
fn highlight_color(highlight: Highlight) -> Option<Color> {
    Some(match highlight {
        Highlight::Normal => return None,
        Highlight::Comment | Highlight::MultilineComment => Color::DarkCyan,
        Highlight::Keyword1 => Color::DarkYellow,
        Highlight::Keyword2 => Color::DarkGreen,
        Highlight::String => Color::DarkMagenta,
        Highlight::Number => Color::DarkRed,
    })
}

/// Draws the reverse-video status bar on the second-to-last row.
fn draw_status_bar(
    stdout: &mut io::Stdout,
    editor: &Editor,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let document = editor.document();
    let (_, cy) = editor.cursor();

    let modified = if document.is_dirty() {
        " (modified)"
    } else {
        ""
    };
    let left = truncate(
        &format!(
            "{} - {} lines{}",
            document.filename().unwrap_or("[No Name]"),
            document.len(),
            modified
        ),
        cols,
    );
    let right = truncate(
        &format!("{} | {}/{}", document.filetype(), cy + 1, document.len()),
        cols,
    );

    let row = rows.saturating_sub(2);
    queue!(
        stdout,
        MoveTo(0, row),
        SetAttribute(Attribute::Reverse),
        Clear(ClearType::CurrentLine),
        Print(left),
    )?;

    let right_len = u16::try_from(right.chars().count()).expect("status text is clamped to cols");
    queue!(
        stdout,
        MoveTo(cols.saturating_sub(right_len), row),
        Print(right),
        SetAttribute(Attribute::Reset),
    )?;
    Ok(())
}

/// Draws the message bar on the bottom row.
fn draw_message_bar(
    stdout: &mut io::Stdout,
    editor: &Editor,
    cols: u16,
    rows: u16,
) -> io::Result<()> {
    let message = editor.message().unwrap_or("");
    queue!(
        stdout,
        MoveTo(0, rows.saturating_sub(1)),
        Clear(ClearType::CurrentLine),
        Print(truncate(message, cols)),
    )?;
    Ok(())
}

/// Truncates text to the terminal width (by chars, not bytes).
fn truncate(text: &str, width: u16) -> String {
    text.chars().take(width as usize).collect()
}

fn welcome_line(cols: u16) -> String {
    let text = format!("~ {NAME} editor -- version {VERSION}");
    truncate(&text, cols)
}
