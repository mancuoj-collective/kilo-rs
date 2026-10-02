//! `kilo` binary entry point: setup + call into the library; no business logic.

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    // Must come after color_eyre::install() to wrap the panic hook it sets.
    kilo::tui::install_panic_hook();
    kilo::run()
}
