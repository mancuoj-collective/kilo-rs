use anyhow::{Context, Result};

mod app;
mod key;
mod terminal;

fn main() -> Result<()> {
    let mut editor = app::Editor::new().context("failed to initialize editor")?;
    editor.run().context("editor stopped unexpectedly")
}
