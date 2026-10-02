//! Terminal lifecycle management.
//!
//! Wraps the must-be-paired operations (raw mode, alternate screen, cursor) in a
//! [`Terminal`] guard, and restores the terminal on panic via [`install_panic_hook`].

use std::{io, panic};

use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

/// Guard for full-screen editing state.
///
/// While held, the terminal is in raw mode, on the alternate screen, with the cursor
/// hidden; dropping it restores the terminal — on a normal return, an early `?`
/// return, or a panic unwind.
pub struct Terminal;

impl Terminal {
    /// Enters full-screen editing state.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`io::Error`] if the terminal is unavailable.
    pub fn enter() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        // Build the guard first; if the command below fails, `guard` drops and raw
        // mode is restored.
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

/// Installs a panic hook that restores the terminal, then hands off to the previous
/// hook.
///
/// Must be called *after* other hooks (e.g. `color_eyre::install`) are set.
pub fn install_panic_hook() {
    let prev = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        restore();
        prev(info);
    }));
}

/// Restores the terminal. Shared by `Drop` and the panic hook.
fn restore() {
    let mut stdout = io::stdout();
    let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}
