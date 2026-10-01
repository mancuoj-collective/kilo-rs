pub mod tui;

use std::{
    env, fs,
    io::{self, Write},
};

use color_eyre::Result;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::Print,
    terminal::{self, Clear, ClearType},
};

use crate::tui::Terminal;

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

    let _guard = Terminal::enter()?;
    draw(&lines)?;

    loop {
        match event::read()? {
            Event::Key(key) if key.is_press() => match key.code {
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                _ => {}
            },
            Event::Resize(_, _) => draw(&lines)?,
            _ => {}
        }
    }

    Ok(())
}

fn draw(lines: &[String]) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let mut stdout = io::stdout();

    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..rows {
        if let Some(line) = lines.get(y as usize) {
            queue!(stdout, Print(truncate(line, cols)))?;
        } else if lines.is_empty() && y == rows / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
        }

        if y + 1 < rows {
            queue!(stdout, Print("\r\n"))?;
        }
    }

    queue!(stdout, MoveTo(0, 0))?;
    stdout.flush()
}

fn welcome_line(cols: u16) -> String {
    let text = format!("~ {NAME} editor -- version {VERSION}");
    truncate(&text, cols)
}

fn truncate(text: &str, width: u16) -> String {
    text.chars().take(width as usize).collect()
}
