use crate::RI_USER_AGENT;
use anyhow::Error;
use reqwest::{header::USER_AGENT, Client};
use semver::Version;
use serde::Deserialize;

const BASE: &str = "https://crates.io/api/v1/crates/rustinsight";

pub struct CratesApi {
    client: Client,
}

impl CratesApi {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn fetch_info(&mut self) -> Result<CratesInfo, Error> {
        let url = format!("{BASE}");
        let info = self
            .client
            .get(&url)
            .header(USER_AGENT, RI_USER_AGENT)
            .send()
            .await?
            .json()
            .await?;
        Ok(info)
    }

    pub async fn latest_version(&mut self) -> Result<Version, Error> {
        self.fetch_info()
            .await?
            .versions
            .into_iter()
            .next()
            .map(|remote| remote.num)
            .ok_or_else(|| Error::msg("No versions avaialble"))
    }
}

#[derive(Debug, Deserialize)]
pub struct CratesInfo {
    pub versions: Vec<CratesVersion>,
}

#[derive(Debug, Deserialize)]
pub struct CratesVersion {
    pub num: Version,
}
