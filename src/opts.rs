use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
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
    Update,
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
