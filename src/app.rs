use anyhow::Result;

use crate::{
    key::{self, Key},
    terminal,
};

pub struct Editor {
    _raw_mode: terminal::RawMode,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            _raw_mode: terminal::enter()?,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            if matches!(key::read()?, Key::Ctrl('q')) {
                return Ok(());
            }
        }
    }
}
