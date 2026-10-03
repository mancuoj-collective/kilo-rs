//! kilo-rs: a text editor rewritten from kilo in Rust.
//!
//! Logic lives in the library (so integration tests can reach it); `main.rs` is a thin
//! shell. Terminal lifecycle is in [`tui`], the buffer in [`document`], cursor and
//! viewport in [`editor`], and rendering in [`ui`].

pub mod document;
pub mod editor;
pub mod row;
pub mod tui;
pub mod ui;

use std::{env, fs, io};

use color_eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};

use crate::{
    editor::{Editor, Move},
    tui::Terminal,
};

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
    render(&mut editor)?;

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
        render(&mut editor)?;
    }

    Ok(())
}

/// Scrolls the viewport to the current terminal size, then draws one frame.
fn render(editor: &mut Editor) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let text_height = (rows as usize).saturating_sub(1);
    editor.scroll(text_height, cols as usize);
    ui::draw(editor, cols, rows)
}
