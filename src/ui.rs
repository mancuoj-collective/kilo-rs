//! Rendering: turn the editor state into terminal output.
//!
//! This layer is pure with respect to the editor: it only reads it and writes to the
//! terminal. Scrolling the viewport is *not* done here — the caller does that before
//! calling [`draw`], so rendering stays a function of the current state.

use std::io::{self, Write};

use crossterm::{
    cursor::{self, MoveTo},
    queue,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::editor::Editor;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Draws one frame: the buffer, a status line, and the editor cursor.
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
    let text_height = height.saturating_sub(1); // keep the bottom row for the status line

    let (coloff, rowoff) = editor.viewport();
    let (rx, cy) = editor.cursor();
    let document = editor.document();

    let mut stdout = io::stdout();
    queue!(stdout, cursor::Hide, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..text_height {
        let filerow = rowoff + y;
        if let Some(row) = document.row(filerow) {
            queue!(stdout, Print(row.render_window(coloff, width)))?;
        } else if document.is_empty() && y == text_height / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
        }

        if y + 1 < height {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    // Status line on the bottom row.
    let name = document.filename().unwrap_or("[No Name]");
    let modified = if document.is_dirty() {
        "  [modified]"
    } else {
        ""
    };
    let status = truncate(&format!("{name}{modified}"), cols);
    queue!(
        stdout,
        MoveTo(0, rows.saturating_sub(1)),
        Clear(ClearType::CurrentLine),
        Print(status),
    )?;

    // The viewport guarantees the cursor is on screen, so these conversions can't fail.
    let cursor_col =
        u16::try_from(rx - coloff).expect("cursor column should be inside the viewport");
    let cursor_row = u16::try_from(cy - rowoff).expect("cursor row should be inside the viewport");
    queue!(stdout, MoveTo(cursor_col, cursor_row), cursor::Show)?;
    stdout.flush()
}

/// Truncates text to the terminal width (by chars, not bytes).
fn truncate(text: &str, width: u16) -> String {
    text.chars().take(width as usize).collect()
}

fn welcome_line(cols: u16) -> String {
    let text = format!("~ {NAME} editor -- version {VERSION}");
    truncate(&text, cols)
}
