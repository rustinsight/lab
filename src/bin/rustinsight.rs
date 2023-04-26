use anyhow::Error;
use clap::Parser;
use colored::Colorize;
use knowledge::app::App;
use knowledge::opts::Opts;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Opts::parse();
    if let Err(err) = App::entrypoint(opts, true).await {
        println!("Failed: {}", err.to_string().red());
    }
    Ok(())
}
