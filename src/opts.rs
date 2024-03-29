use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub command: Option<AppCommand>,
}

#[derive(Debug, Subcommand)]
pub enum AppCommand {
    /// Removes the cache folder
    Clean,
    /// Launches the Ri! Learn app
    Learn,
    /// Updates the launcher and apps
    Update(UpdateCommand),
    /*
    /// Opens a link to the latest Stack version
    Stack,
    */
}

impl Default for AppCommand {
    fn default() -> Self {
        Self::Learn
    }
}

#[derive(Debug, Parser, Clone)]
pub struct UpdateCommand {
    /// Override an operating system of downloading assets
    #[clap(long, short)]
    pub system: Option<String>,
    /// Reload assets
    #[clap(long, short)]
    pub force: bool,
}
