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
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal,
};

use crate::{
    editor::{Editor, Find, Move},
    tui::Terminal,
};

/// Number of Ctrl-Q presses needed to quit with unsaved changes.
const QUIT_TIMES: u8 = 3;

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
    editor.set_message("HELP: Ctrl-S = save | Ctrl-F = find | Ctrl-Q = quit");

    let _guard = Terminal::enter()?;
    render(&mut editor)?;

    let mut quit_times = QUIT_TIMES;
    loop {
        let Event::Key(key) = event::read()? else {
            render(&mut editor)?;
            continue;
        };
        if key.is_press() && handle_key(&mut editor, key, &mut quit_times)? {
            break;
        }
        render(&mut editor)?;
    }

    Ok(())
}

/// Handles one key press. Returns `true` when the editor should quit.
fn handle_key(editor: &mut Editor, key: KeyEvent, quit_times: &mut u8) -> io::Result<bool> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char('q') if ctrl => {
            if editor.document().is_dirty() && *quit_times > 0 {
                let time = if *quit_times == 1 { "time" } else { "times" };
                editor.set_message(format!(
                    "WARNING! Unsaved changes. Press Ctrl-Q {quit_times} more {time} to quit."
                ));
                *quit_times -= 1;
                return Ok(false);
            }
            return Ok(true);
        }
        KeyCode::Char('s') if ctrl => save(editor)?,
        KeyCode::Char('f') if ctrl => find(editor)?,
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
    *quit_times = QUIT_TIMES;
    Ok(false)
}

/// Ctrl-S: prompt for a file name when needed, then write the file.
fn save(editor: &mut Editor) -> io::Result<()> {
    if editor.document().filename().is_none() {
        let Some(name) = prompt(editor, "Save as: ")? else {
            editor.set_message("Save aborted");
            return Ok(());
        };
        editor.set_filename(name);
    }
    let bytes = editor.save()?;
    editor.set_message(format!("{bytes} bytes written to disk"));
    Ok(())
}

/// Ctrl-F: live search; `Esc` restores the cursor, `Enter` keeps it.
fn find(editor: &mut Editor) -> io::Result<()> {
    let saved = editor.view_state();
    let query = prompt_with(editor, "Search: ", |editor, query, key| match key {
        KeyCode::Enter | KeyCode::Esc => {}
        KeyCode::Up | KeyCode::Left => editor.find(query, Find::Prev),
        KeyCode::Down | KeyCode::Right => editor.find(query, Find::Next),
        _ => editor.find(query, Find::FromStart),
    })?;
    editor.clear_matches();
    if query.is_none() {
        editor.restore_view(saved);
    }
    Ok(())
}

/// Reads a line from the message bar. `Esc` cancels, `Enter` confirms.
fn prompt(editor: &mut Editor, label: &str) -> io::Result<Option<String>> {
    prompt_with(editor, label, |_, _, _| {})
}

/// Like [`prompt`], but calls `on_key` after every key press (used by live search).
fn prompt_with(
    editor: &mut Editor,
    label: &str,
    mut on_key: impl FnMut(&mut Editor, &str, KeyCode),
) -> io::Result<Option<String>> {
    let mut input = String::new();
    loop {
        editor.set_message(format!("{label}{input}"));
        render(editor)?;

        let Event::Key(key) = event::read()? else {
            continue;
        };
        if !key.is_press() {
            continue;
        }

        match key.code {
            KeyCode::Esc => {
                on_key(editor, &input, KeyCode::Esc);
                editor.clear_message();
                return Ok(None);
            }
            KeyCode::Enter => {
                if !input.is_empty() {
                    on_key(editor, &input, KeyCode::Enter);
                    editor.clear_message();
                    return Ok(Some(input));
                }
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(ch) if !key.modifiers.contains(KeyModifiers::CONTROL) => input.push(ch),
            _ => {}
        }
        on_key(editor, &input, key.code);
    }
}

/// Scrolls the viewport to the current terminal size, then draws one frame.
fn render(editor: &mut Editor) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let text_height = (rows as usize).saturating_sub(2);
    editor.scroll(text_height, cols as usize);
    ui::draw(editor, cols, rows)
}
