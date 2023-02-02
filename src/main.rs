use anyhow::Error;
use clap::Parser;
use colored::Colorize;
use rustinsight::app::App;
use rustinsight::opts::Opts;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Opts::parse();
    if let Err(err) = App::entrypoint(opts).await {
        println!("Failed: {}", err.to_string().red());
    }
    Ok(())
}
