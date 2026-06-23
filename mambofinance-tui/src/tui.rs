use color_eyre::Result;
use ratatui::DefaultTerminal;

pub fn init() -> Result<DefaultTerminal> {
    Ok(ratatui::init())
}

pub fn restore() {
    ratatui::restore();
}
