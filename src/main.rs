use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    kilo::tui::install_panic_hook();
    kilo::run()
}
