use crate::app_info::AppInfo;
use crate::built_info;
use crate::RI_USER_AGENT;
use anyhow::Error;
use futures::StreamExt;
use indicatif::ProgressBar;
use reqwest::header::{CONTENT_LENGTH, USER_AGENT};
use reqwest::Client;
use semver::Version;
use serde::Deserialize;
use tempfile::tempfile;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};

pub struct GitHubApi {
    client: Client,
}

impl GitHubApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn releases(&mut self, url: &str) -> Result<Vec<Release>, Error> {
        let releases = self
            .client
            .get(url)
            .header(USER_AGENT, RI_USER_AGENT)
            .send()
            .await?
            .json()
            .await?;
        Ok(releases)
    }

    pub async fn latest_release(&mut self, app_info: &AppInfo) -> Result<Release, Error> {
        let latest_release = self
            .releases(&app_info.link)
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| Error::msg("No releases available"))?;
        Ok(latest_release)
    }

    pub async fn download_assets(&mut self, url: &str) -> Result<File, Error> {
        let resp = self
            .client
            .get(url)
            .header(USER_AGENT, RI_USER_AGENT)
            .send()
            .await?;
        let total = resp
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or_else(|| Error::msg("Can't detect size of assets"))?
            .to_str()?
            .parse()?;
        let mut chunks = resp.bytes_stream();
        let mut archive = File::from_std(tempfile()?);
        let bar = ProgressBar::new(total);
        while let Some(chunk) = chunks.next().await.transpose()? {
            bar.inc(chunk.len() as u64);
            archive.write_all(&chunk).await?;
        }
        bar.finish();
        archive.seek(SeekFrom::Start(0)).await?;
        Ok(archive)
    }
}

#[derive(Debug, Deserialize)]
pub struct Release {
    /// Title
    pub name: String,
    pub html_url: String,
    /// IMPORTANT: `Tag` has to be a valid semver
    #[serde(rename = "tag_name")]
    pub version: Version,
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn get_asset_for_os(
        &self,
        app_info: &AppInfo,
        system: Option<&str>,
    ) -> Result<&str, Error> {
        let os = system.unwrap_or(built_info::CFG_OS);
        let arch = built_info::CFG_TARGET_ARCH;
        let ver = &self.version;
        let name = &app_info.name;
        let expected_asset = format!("{name}-{ver}-{os}-{arch}.tar.gz");
        for asset in &self.assets {
            if asset.name == expected_asset {
                return Ok(&asset.browser_download_url);
            }
        }
        Err(Error::msg("Assets for the current OS was not found"))
    }
}

#[derive(Debug, Deserialize)]
pub struct Asset {
    /// The name of the file.
    pub name: String,
    pub browser_download_url: String,
}
