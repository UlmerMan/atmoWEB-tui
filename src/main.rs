use atmoweb_tui::app::App;
use atmoweb_tui::cli;

use std::error::Error;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = cli::Args::parse();

    let mut terminal = ratatui::init();
    let mut app = App::new(args.address);

    let result = app.run(&mut terminal).await;
    ratatui::restore();
    result
}
