use anyhow::Result;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode, size};
use crossterm::{Command, queue};
use std::io::{Write, stdout};

#[derive(Debug, Default, Clone, Copy)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

pub struct Terminal;

impl Terminal {
    pub fn initialize() -> Result<()> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::flush()
    }

    pub fn terminate() -> Result<()> {
        disable_raw_mode()?;
        Self::flush()
    }

    pub fn clear_screen() -> Result<()> {
        Self::queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> Result<()> {
        Self::queue_command(Clear(ClearType::CurrentLine))
    }

    pub fn move_caret_to(position: Position) -> Result<()> {
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))
    }

    pub fn hide_caret() -> Result<()> {
        Self::queue_command(Hide)
    }

    pub fn show_caret() -> Result<()> {
        Self::queue_command(Show)
    }

    pub fn print(string: &str) -> Result<()> {
        Self::queue_command(Print(string))
    }

    pub fn size() -> Result<Size> {
        let (width_u16, height_u16) = size()?;
        let height = height_u16 as usize;
        let width = width_u16 as usize;
        Ok(Size { height, width })
    }

    pub fn flush() -> Result<()> {
        stdout().flush()?;
        Ok(())
    }

    fn queue_command<T: Command>(command: T) -> Result<()> {
        queue!(stdout(), command)?;
        Ok(())
    }
}
