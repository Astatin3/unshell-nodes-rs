mod app;
mod tabs;

use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = app::App::default().run(terminal);
    ratatui::restore();
    app_result
}
