mod app;
mod tui;
mod ui;
mod update;
mod widgets;

use app::App;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = tui::init()?;
    let result = App::new("USER")?.run(terminal);
    tui::restore();
    result
}
