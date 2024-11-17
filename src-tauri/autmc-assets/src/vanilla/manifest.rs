use super::deserializers;
use crate::consts::VANILLA_MANIFEST_URL;
use indexmap::IndexMap;
use log::info;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
/// Struct holding everything returned in the vanilla manifest json.
pub struct VanillaManifest {
    // latest: VanillaLatest,
    #[serde(deserialize_with = "deserializers::as_version_map")]
    pub versions: IndexMap<String, VanillaManifestVersion>,
}

#[derive(Debug, Deserialize)]
/// The version metadata returned in the manifest request.
pub struct VanillaManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    // time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    // #[serde(rename = "complianceLevel")]
    // compliance_level: u32,
}

pub async fn download_vanilla_manifest() -> reqwest::Result<VanillaManifest> {
    info!("Downloading vanilla manifest");
    let client = reqwest::Client::new();
    let vanilla_response = client.get(VANILLA_MANIFEST_URL).send().await?;
    vanilla_response.json::<VanillaManifest>().await
}
