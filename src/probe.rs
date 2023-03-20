use anyhow::{anyhow as err, Error};
use reqwest::Client;
use tokio::time::{sleep, Duration};

pub struct ProbeTool {
    client: Client,
}

impl ProbeTool {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn probe(&self, url: &str) -> Result<(), Error> {
        for _ in 0..10 {
            if self.client.get(url).send().await.is_ok() {
                return Ok(());
            } else {
                sleep(Duration::from_millis(500)).await;
            }
        }
        Err(err!("The app is still not available at: {url}"))
    }

    pub async fn probe_is_free(&self, url: &str) -> Result<(), Error> {
        if self.client.get(url).send().await.is_err() {
            Ok(())
        } else {
            Err(err!("The port is not free: {url}"))
        }
    }
}
