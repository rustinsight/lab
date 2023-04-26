use anyhow::Error;
use clap::Parser;
use colored::Colorize;
use rustinsight::app::App;
use rustinsight::opts::Opts;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts = Opts::parse();
    println!("Install the new version of the launcher app is available:");
    let command = "cargo install knowledge --force".red();
    println!("$ {command}");
    println!("");
    if let Err(err) = App::entrypoint(opts, false).await {
        let err = err.to_string().red();
        println!("Failed: {err}",);
        let site = "https://deniskolodin.com/troubleshooting".green();
        println!("Check details here: {site}");
    }
    Ok(())
}
