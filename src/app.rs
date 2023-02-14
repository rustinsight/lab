use crate::app_info::{self, AppInfo, Color};
use crate::cacher::{AppState, Cacher};
use crate::opts::{AppCommand, Opts};
use crate::{crates::CratesApi, github::GitHubApi};
use anyhow::Error;
use colored::Colorize;
use dialoguer::Confirm;
use flate2::read::GzDecoder;
use std::process::Stdio;
use tar::Archive;
use tokio::process::{Child, Command};
use tokio::{select, signal};

pub struct App {
    cacher: Cacher,
    crates_api: CratesApi,
    github_api: GitHubApi,
    app: Option<Child>,
}

impl App {
    pub async fn entrypoint(opts: Opts, ride: bool) -> Result<(), Error> {
        let mut app = Self::init().await?;
        match opts.command {
            None => {
                app.command_update(false).await?;
                app.command_learn(ride).await?;
            }
            Some(AppCommand::Update) => {
                app.command_update(true).await?;
            }
            Some(AppCommand::Learn) => {
                app.command_learn(ride).await?;
            }
            /*
            Some(AppCommand::Stack) => {
                app.command_stack().await?;
            }
            */
            Some(AppCommand::Clean) => {
                app.command_clean().await?;
            }
        }
        Ok(())
    }

    async fn init() -> Result<Self, Error> {
        let mut cacher = Cacher::create().await?;
        cacher.initialize().await?;
        let crates_api = CratesApi::new();
        let github_api = GitHubApi::new();
        Ok(Self {
            cacher,
            crates_api,
            github_api,
            app: None,
        })
    }

    async fn update_launcher(&mut self, force: bool) -> Result<(), Error> {
        if self.cacher.launcher.is_update_required() || force {
            println!("Checking an update for the launcher...");
            let version = self.crates_api.latest_version().await?;

            if self.cacher.launcher.is_outdated(version) {
                println!(
                    "New version of launcher is available. To update run the following command"
                );
                let command = "cargo install rustinsight".green();
                println!("{command}");
            }

            self.cacher.launcher.update_check();
            self.cacher.write_state().await?;
        }
        Ok(())
    }

    async fn update_ri_learn(&mut self, force: bool) -> Result<(), Error> {
        if self.cacher.ri_learn.is_update_required() || force {
            println!("Checking an update for the app...");

            let latest = self.github_api.latest_release(&app_info::LEARN).await?;
            let version = latest.version.clone();
            if self.cacher.ri_learn.is_outdated(version) {
                println!("Downloading {}...", latest.version);
                let url = latest.get_asset_for_os(&app_info::LEARN)?;
                let tar_gz = self.github_api.download_assets(url).await?.into_std().await;
                println!("Unpacking...");
                let tar = GzDecoder::new(tar_gz);
                let mut archive = Archive::new(tar);
                archive.unpack(self.cacher.bin_dir())?;
                self.cacher.fix_binaries().await?;
                self.cacher.ri_learn.version = Some(latest.version);
                println!("Done");
            }

            // Never called if update has failed
            self.cacher.ri_learn.update_check();
            self.cacher.write_state().await?;
        }
        Ok(())
    }

    /*
    async fn update_ri_stack(&mut self, force: bool) -> Result<(), Error> {
        if self.cacher.ri_stack.is_update_required() || force {
            println!("Checking an update for the stack...");

            let latest = self.github_api.latest_release(&app_info::STACK).await?;
            let version = latest.version.clone();
            if self.cacher.ri_stack.is_outdated(version) {
                self.cacher.ri_stack.version = Some(latest.version);
                println!("The latest stack release: {}", latest.html_url.green());
            }
            self.cacher.ri_stack.update_check();
            self.cacher.write_state().await?;
        }
        Ok(())
    }
    */

    fn start_app(&mut self, app_info: &AppInfo, args: Vec<String>) -> Result<(), Error> {
        let mut bin_path = self.cacher.bin_dir().clone();
        bin_path.push(app_info.name);
        let child = Command::new(bin_path)
            .args(args)
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        self.app = Some(child);
        Ok(())
    }

    async fn closed(&mut self) -> Result<(), Error> {
        if let Some(child) = self.app.as_mut() {
            child.wait().await?;
        }
        Ok(())
    }

    async fn terminate_app(&mut self) -> Result<(), Error> {
        let mut child = self
            .app
            .take()
            .ok_or_else(|| Error::msg("App is not sarted"))?;
        child.kill().await?;
        child.wait().await?;
        Ok(())
    }

    fn show_banner(&self, logo: &AppInfo, app_state: &AppState) -> Result<(), Error> {
        let launcher_ver = self.cacher.launcher.get_version()?;
        let app_ver = app_state.get_version()?;
        let Color(r, g, b) = logo.bg;
        let ri = logo.line_1.bold().white().on_truecolor(r, g, b);
        println!("{ri} v{app_ver} (product)");
        let name = logo.line_2.bold().white().on_truecolor(r, g, b);
        let launcher_info = format!("v{launcher_ver} (launcher)").truecolor(100, 100, 100);
        println!("{name} {launcher_info}");
        println!("");
        Ok(())
    }

    pub async fn command_update(&mut self, force: bool) -> Result<(), Error> {
        if let Err(err) = self.update_launcher(force).await {
            let err = err.to_string().red();
            println!("Launcher updating failed: {err}");
        }
        if let Err(err) = self.update_ri_learn(force).await {
            let err = err.to_string().red();
            println!("App updating failed: {err}");
        }
        /*
        if let Err(err) = self.update_ri_stack(force).await {
            let err = err.to_string().red();
            println!("Stack updating failed: {err}");
        }
        */
        Ok(())
    }

    /*
    pub async fn command_stack(&mut self) -> Result<(), Error> {
        self.show_banner(&app_info::STACK, &self.cacher.ri_stack)?;
        Ok(())
    }
    */

    pub async fn command_learn(&mut self, ride: bool) -> Result<(), Error> {
        self.show_banner(&app_info::LEARN, &self.cacher.ri_learn)?;

        let url = "http://localhost:6361/";
        let link = url.green(); //.truecolor(255, 61, 0);
        println!("The app is started and active at: {link}");
        println!("Keep this terminal active to use the app.");
        println!("");

        let workdir = std::env::current_dir()?;
        let workdir = workdir.display().to_string().green();
        println!("Working folder is: {workdir}");

        let mut args = Vec::new();
        if ride {
            args.push("--ride".into());
        }
        self.start_app(&app_info::LEARN, args)?;
        webbrowser::open(url).ok();
        select! {
            _ = signal::ctrl_c() => {
                println!("Terminating the app.");
                self.terminate_app().await?;
            }
            _ = self.closed() => {
                println!("App was closed. Done.");
            }
        }
        Ok(())
    }

    pub async fn command_clean(self) -> Result<(), Error> {
        if Confirm::new()
            .with_prompt("Do you want to clean the cache?")
            .interact()?
        {
            println!("Cleaning the cache...");
            self.cacher.remove_cache().await?;
        }
        Ok(())
    }
}
