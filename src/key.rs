use anyhow::{Context, Result};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Ctrl(char),
    Char(char),
    Other,
}

pub fn read() -> Result<Key> {
    event::read()
        .context("failed to read terminal event")
        .map(Key::from_event)
}

impl Key {
    fn from_event(event: Event) -> Self {
        let Event::Key(key_event) = event else {
            return Self::Other;
        };

        match key_event.code {
            KeyCode::Char(ch) if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                Self::Ctrl(ch)
            }
            KeyCode::Char(ch) => Self::Char(ch),
            _ => Self::Other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent};

    #[test]
    fn maps_ctrl_q() {
        let event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::CONTROL));
        assert_eq!(Key::from_event(event), Key::Ctrl('q'));
    }
}
