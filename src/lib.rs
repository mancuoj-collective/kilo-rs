pub mod editor;
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

pub fn run() -> Result<()> {
    let lines = match env::args().nth(1) {
        Some(path) => fs::read_to_string(&path)?
            .lines()
            .map(str::to_owned)
            .collect(),
        None => Vec::new(),
    };

    let mut editor = Editor::new(lines);

    let _guard = Terminal::enter()?;
    draw(&mut editor)?;

    loop {
        match event::read()? {
            Event::Key(key) if key.is_press() => match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Up => editor.move_cursor(Move::Up),
                KeyCode::Down => editor.move_cursor(Move::Down),
                KeyCode::Left => editor.move_cursor(Move::Left),
                KeyCode::Right => editor.move_cursor(Move::Right),
                KeyCode::Home => editor.move_cursor(Move::Home),
                KeyCode::End => editor.move_cursor(Move::End),
                _ => {}
            },
            _ => {}
        }
        draw(&mut editor)?;
    }

    Ok(())
}

fn draw(editor: &mut Editor) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let width = cols as usize;
    let height = rows as usize;

    editor.scroll(height, width);

    let (coloff, rowoff) = editor.viewport();
    let (cx, cy) = editor.cursor();
    let lines = editor.lines();

    let mut stdout = io::stdout();
    queue!(stdout, cursor::Hide, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..height {
        let filerow = rowoff + y;
        if let Some(line) = lines.get(filerow) {
            let text: String = line.chars().skip(coloff).take(width).collect();
            queue!(stdout, Print(text))?;
        } else if lines.is_empty() && y == height / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
        }

        if y + 1 < height {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    let cursor_col = u16::try_from(cx - coloff).expect("光标列应落在视口内");
    let cursor_row = u16::try_from(cy - rowoff).expect("光标行应落在视口内");
    queue!(stdout, MoveTo(cursor_col, cursor_row), cursor::Show)?;
    stdout.flush()
}

fn welcome_line(cols: u16) -> String {
    let text = format!("~ {NAME} editor -- version {VERSION}");
    truncate(&text, cols)
}

fn truncate(text: &str, width: u16) -> String {
    text.chars().take(width as usize).collect()
}
