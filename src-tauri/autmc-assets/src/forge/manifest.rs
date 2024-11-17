use crate::consts::FORGE_MANIFEST_URL;
use log::info;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct ForgeManifest(pub HashMap<String, Vec<String>>);

pub async fn download_forge_manifest() -> reqwest::Result<ForgeManifest> {
    info!("Downloading forge manifest");
    let client = reqwest::Client::new();
    let forge_response = client.get(FORGE_MANIFEST_URL).send().await?;
    forge_response.json::<ForgeManifest>().await
}
