use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::Print,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use std::io;

pub struct RawMode;

impl RawMode {
    fn new() -> Result<Self> {
        enable_raw_mode().context("failed to enable raw mode")?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), Show, Clear(ClearType::All), MoveTo(0, 0));
    }
}

pub fn enter() -> Result<RawMode> {
    let raw_mode = RawMode::new()?;

    execute!(
        io::stdout(),
        Clear(ClearType::All),
        MoveTo(0, 0),
        Print("Kilo Rust - press Ctrl-Q to quit\r\n"),
        Hide
    )
    .context("failed to initialize terminal")?;

    Ok(raw_mode)
}
