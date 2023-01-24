use serde::Deserialize;

use crate::{consts::FABRIC_BASE_URL, web_services::downloader::download_json_object, state::resource_manager::ManifestResult};

use super::vanilla::LaunchArguments;

#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    separator: String,
    build: i32,
    maven: String,
    pub version: String,
    stable: bool
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderManifest(pub Vec<FabricLoaderVersion>);

#[derive(Debug, Deserialize)]
pub struct FabricLibrary {
    name: String,
    url: String
}

#[derive(Debug, Deserialize)]
pub struct FabricProfile {
    id: String,
    #[serde(rename = "inheritsFrom")]
    inherits_from: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    time: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    arguments: LaunchArguments,
    libraries: Vec<FabricLibrary>
}

pub async fn download_fabric_profile(minecraft_version: &str, fabric_version: &str) -> ManifestResult<FabricProfile> {
    let url = format!("{}/versions/loader/{}/{}/profile/json", FABRIC_BASE_URL, minecraft_version, fabric_version);
    Ok(download_json_object::<FabricProfile>(&url).await?)
}