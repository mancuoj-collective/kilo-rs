//! kilo-rs: a text editor rewritten from kilo in Rust.
//!
//! Logic lives in the library (so integration tests can reach it); `main.rs` is a thin
//! shell. Terminal lifecycle is in [`tui`], editor state in [`editor`]; rendering and the
//! event loop stay here for now, to split into `ui` / `app` later.

pub mod editor;
pub mod row;
pub mod tui;

use std::{
    env, fs,
    io::{self, Write},
};

use color_eyre::Result;
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::{
    editor::{Editor, Move},
    tui::Terminal,
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Runs the editor: read the file, take over the terminal, enter the event loop.
///
/// # Errors
///
/// Returns an error if reading the file or a terminal operation fails.
pub fn run() -> Result<()> {
    // Read the file name from argv; with no argument, start with an empty buffer.
    let path = env::args().nth(1);
    let lines = match &path {
        Some(path) => fs::read_to_string(path)?
            .lines()
            .map(str::to_owned)
            .collect(),
        None => Vec::new(),
    };

    let mut editor = Editor::new(&lines, path);

    let _guard = Terminal::enter()?;
    draw(&mut editor)?;

    loop {
        match event::read()? {
            Event::Key(key) if key.is_press() => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                match key.code {
                    KeyCode::Char('q') if ctrl => break,
                    KeyCode::Char('s') if ctrl => editor.save()?,
                    KeyCode::Char(ch) if !ctrl => editor.insert_char(ch),
                    KeyCode::Enter => editor.insert_newline(),
                    KeyCode::Backspace | KeyCode::Delete => editor.delete_char(),
                    KeyCode::Up => editor.move_cursor(Move::Up),
                    KeyCode::Down => editor.move_cursor(Move::Down),
                    KeyCode::Left => editor.move_cursor(Move::Left),
                    KeyCode::Right => editor.move_cursor(Move::Right),
                    KeyCode::Home => editor.move_cursor(Move::Home),
                    KeyCode::End => editor.move_cursor(Move::End),
                    _ => {}
                }
            }
            _ => {}
        }
        draw(&mut editor)?;
    }

    Ok(())
}

/// Draws one frame: render the file at the viewport offset, then place the terminal
/// cursor at the editor cursor.
///
/// Calls [`Editor::scroll`] first, hence the `&mut Editor`.
fn draw(editor: &mut Editor) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let width = cols as usize;
    let height = rows as usize;
    let text_height = height.saturating_sub(1); // keep the bottom row for the status line

    editor.scroll(text_height, width);

    let (coloff, rowoff) = editor.viewport();
    let (rx, cy) = editor.cursor();

    let mut stdout = io::stdout();
    queue!(stdout, cursor::Hide, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..text_height {
        let filerow = rowoff + y;
        if let Some(text) = editor.render_line(filerow, coloff, width) {
            queue!(stdout, Print(text))?;
        } else if editor.is_empty() && y == text_height / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
        }

        if y + 1 < height {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    // Status line on the bottom row.
    let name = editor.filename().unwrap_or("[No Name]");
    let modified = if editor.is_dirty() {
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

fn welcome_line(cols: u16) -> String {
    let text = format!("~ {NAME} editor -- version {VERSION}");
    truncate(&text, cols)
}

/// Truncates text to the terminal width (by chars, not bytes).
fn truncate(text: &str, width: u16) -> String {
    text.chars().take(width as usize).collect()
}
