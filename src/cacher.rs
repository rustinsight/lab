use crate::{app_info, built_info, VERSION};
use anyhow::Error;
use chrono::{DateTime, Duration, Utc};
use derive_more::{Deref, DerefMut};
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;
use tokio::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct GlobalConfig {
    pub system: String,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            system: built_info::CFG_OS.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LauncherConfig {
    pub global: GlobalConfig,
    pub launcher: AppState,
    pub ri_learn: AppState,
    pub ri_stack: AppState,
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig::default(),
            launcher: AppState {
                version: Some(VERSION.clone()),
                last_check: None,
            },
            ri_learn: AppState {
                version: None,
                last_check: None,
            },
            ri_stack: AppState {
                version: None,
                last_check: None,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AppState {
    pub version: Option<Version>,
    pub last_check: Option<DateTime<Utc>>,
}

impl AppState {
    pub fn is_update_required(&self) -> bool {
        self.is_not_exist() || self.is_allowed_to_check()
    }

    pub fn is_not_exist(&self) -> bool {
        self.version.is_none()
    }

    pub fn is_allowed_to_check(&self) -> bool {
        let deadline = Utc::now() - Duration::days(1);
        match &self.last_check {
            None => true,
            Some(last) if last <= &deadline => true,
            Some(_recent) => false,
        }
    }

    pub fn reset(&mut self) {
        self.version.take();
        self.last_check.take();
    }

    pub fn update_check(&mut self) {
        self.last_check = Some(Utc::now());
    }

    pub fn is_outdated(&self, recent_version: Version) -> bool {
        if let Some(current_version) = self.version.as_ref() {
            current_version < &recent_version
        } else {
            true
        }
    }

    pub fn get_version(&self) -> Result<Version, Error> {
        self.version
            .clone()
            .ok_or_else(|| Error::msg("Version is not available"))
    }
}

#[derive(Debug, Deref, DerefMut)]
pub struct Cacher {
    cache_dir: PathBuf,
    bin_dir: PathBuf,
    state_path: PathBuf,
    #[deref]
    #[deref_mut]
    state: LauncherConfig,
}

impl Cacher {
    pub async fn create() -> Result<Self, Error> {
        // Create paths
        let mut cache_dir =
            dirs::cache_dir().ok_or_else(|| Error::msg("Cache directory is not available."))?;
        cache_dir.push("rustinsight");

        let mut bin_dir = cache_dir.clone();
        bin_dir.push("bin");

        let mut state_path = cache_dir.clone();
        state_path.push("launcher.toml");

        let state = LauncherConfig::default();
        Ok(Self {
            cache_dir,
            bin_dir,
            state_path,
            state,
        })
    }

    pub async fn initialize(&mut self) -> Result<(), Error> {
        self.create_dirs().await?;
        if let Err(_err) = self.try_read_state().await {
            // Can't read a config file, it doesn't exist.
        }
        self.repair_config().await?; // In case if something removed
        self.write_state().await?;
        Ok(())
    }

    async fn create_dirs(&mut self) -> Result<(), Error> {
        // Create dirs
        fs::create_dir_all(&self.bin_dir).await?;
        Ok(())
    }

    /// In case if binaries were deleted
    async fn repair_config(&mut self) -> Result<(), Error> {
        // Update launcher's version to the current
        self.launcher.version = Some(VERSION.clone());
        // Checking binaries
        let mut entries = fs::read_dir(&self.bin_dir).await?;
        let app_prefix = &app_info::LEARN.name;
        while let Some(entry) = entries.next_entry().await? {
            if entry.file_name().to_string_lossy().starts_with(app_prefix) {
                return Ok(());
            }
        }
        // File not found, than it has to be downloaded
        self.state.ri_learn.reset();
        Ok(())
    }

    pub fn bin_dir(&self) -> &PathBuf {
        &self.bin_dir
    }

    /// Changes permissions and extension
    pub async fn fix_binaries(&mut self) -> Result<(), Error> {
        let mut entries = fs::read_dir(&self.bin_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let bin_file = File::open(entry.path()).await?;
            // Changes permissions on unix-like systems
            #[cfg(not(target_os = "windows"))]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perm = bin_file.metadata().await?.permissions();
                let new_mode = perm.mode() | 0o511; // adds executable permission `(rx)xx`
                perm.set_mode(new_mode);
                bin_file.set_permissions(perm).await?;
            }
            #[cfg(target_os = "windows")]
            {
                let original_path = entry.path();
                if original_path.extension().is_none() {
                    let path_with_ext = original_path.with_extension("exe");
                    fs::rename(original_path, path_with_ext).await?;
                }
            }
        }
        Ok(())
    }

    async fn try_read_state(&mut self) -> Result<(), Error> {
        let contents = fs::read_to_string(&self.state_path).await?;
        let state = toml::from_str(&contents)?;
        self.state = state;
        Ok(())
    }

    pub async fn write_state(&mut self) -> Result<(), Error> {
        let contents = toml::to_string(&self.state)?;
        fs::write(&self.state_path, contents).await?;
        Ok(())
    }

    pub async fn remove_cache(self) -> Result<(), Error> {
        fs::remove_dir_all(self.cache_dir).await?;
        Ok(())
    }
}
