use crate::client::Cache;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EndpointsYmlEntry {
    pub title: String,
    osm: Option<String>,
    pub wd: Option<String>,
    pub url: String,
    description: Option<String>,
}

/// Loads endpoints.yml with all the known entrypoints
pub async fn get_endpoints<C: Cache>(cache: &C) -> Result<Vec<EndpointsYmlEntry>> {
    if let Some(bytes) = cache.get_inner("endpoints.yml")? {
        return Ok(serde_yaml::from_slice(&bytes)?);
    }

    let endpoints_yml =
        reqwest::get("https://raw.githubusercontent.com/OParl/resources/master/endpoints.yml")
            .await
            .context("Failed to get endpoints.yml")?
            .bytes()
            .await?
            .to_vec();
    cache.set_inner("endpoints.yml", &endpoints_yml)?;
    Ok(serde_yaml::from_slice(&endpoints_yml)?)
}
