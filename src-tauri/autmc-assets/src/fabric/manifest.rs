use log::info;
use serde::Deserialize;

use crate::consts::FABRIC_MANIFEST_URL;


#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    // separator: String,
    // build: i32,
    // maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderManifest(pub Vec<FabricLoaderVersion>);

pub async fn download_fabric_manifest() -> reqwest::Result<FabricLoaderManifest> {
    info!("Downloading fabric manifest");
    let client = reqwest::Client::new();
    let fabric_response = client.get(FABRIC_MANIFEST_URL).send().await?;
    fabric_response.json::<FabricLoaderManifest>().await
}