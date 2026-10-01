use std::io::{self, Write};

use color_eyre::eyre::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    terminal,
};

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    // Drop 里不能 panic，否则会让展开中的 panic 二次 panic，直接 abort。
    // 所以这里显式忽略错误。
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _guard = RawModeGuard::enter()?;

    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };

        if !key.is_press() {
            continue;
        }

        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
            _ => {
                print!("{key:?}\r\n");
                io::stdout().flush()?;
            }
        }
    }

    Ok(())
}
