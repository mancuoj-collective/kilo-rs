use std::{
    env, fs,
    io::{self, Write},
    panic::{set_hook, take_hook},
};

use color_eyre::eyre::Result;
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

struct Terminal;

impl Terminal {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let guard = Self;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        Ok(guard)
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        restore();
    }
}

fn restore() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

fn install_panic_hook() {
    let prev = take_hook();
    set_hook(Box::new(move |info| {
        restore();
        prev(info);
    }));
}

fn draw(lines: &[String]) -> io::Result<()> {
    let (cols, rows) = terminal::size()?;
    let mut stdout = io::stdout();

    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

    for y in 0..rows {
        if y > 0 {
            queue!(stdout, Print("\r\n"))?;
        }

        if let Some(line) = lines.get(y as usize) {
            queue!(stdout, Print(truncate(line, cols)))?;
        } else if lines.is_empty() && y == rows / 3 {
            queue!(stdout, Print(welcome_line(cols)))?;
        } else {
            queue!(stdout, Print('~'))?;
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

fn main() -> Result<()> {
    color_eyre::install()?;
    install_panic_hook();

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
