use std::{
    collections::HashMap,
    io::{BufReader, Cursor, Read},
};

use image::EncodableLayout;
use serde::Deserialize;

use crate::{
    consts::{FORGE_FILES_BASE_URL, FORGE_MAVEN_BASE_URL},
    state::resource_manager::ManifestResult,
    web_services::downloader::{download_bytes_from_url, download_json_object, DownloadResult},
};

use super::vanilla::{LaunchArguments, Library, Logging, VanillaVersion};

#[derive(Debug, Deserialize)]
pub struct ForgeManifest(pub HashMap<String, Vec<String>>);

#[derive(Debug, Deserialize)]
pub struct ForgeHashes {
    pub classifiers: ForgeHashClassifiers,
}

#[derive(Debug, Deserialize)]
pub struct ForgeHashClassifiers {
    sources: ForgeFileHash,
    mdk: ForgeFileHash,       // .zip
    changelog: ForgeFileHash, // .txt
    userdev: ForgeFileHash,
    universal: ForgeFileHash,
    pub installer: ForgeFileHash,
}

// Forge hashes are md5 NOT sha1
#[derive(Debug, Deserialize)]
pub struct ForgeFileHash {
    #[serde(rename = "jar", alias = "txt", alias = "zip")]
    hash: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgeVersion {
    id: String,
    time: String,
    #[serde(rename = "releaseTime")]
    released_time: String,
    #[serde(rename = "type")]
    version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "inheritsFrom")]
    inherits_from: String,
    // FIXME: Ignoring for now since this is just a empty json entry in 1.19.3, not sure about other versions
    // logging: Option<ForgeLogging>,
    arguments: LaunchArguments,
    pub libraries: Vec<Library>,
}

pub async fn download_forge_hashes(forge_version: &str) -> DownloadResult<ForgeHashes> {
    let url = format!("{}/{}/meta.json", FORGE_FILES_BASE_URL, forge_version);
    Ok(download_json_object::<ForgeHashes>(&url).await?)
}

// TODO: Validate jar hash
pub async fn download_forge_version(
    forge_version: &str,
    valid_hash: Option<ForgeFileHash>,
) -> ManifestResult<ForgeVersion> {
    // FIXME: This changes depending on the game version
    // https://github.com/gorilla-devs/GDLauncher/blob/391dd9cc7ef5ac6ef050327abb516eb6799f0539/src/common/reducers/actions.js#L1284
    let terminal = "installer.jar";
    let url = format!(
        "{0}/{1}/forge-{1}-{2}",
        FORGE_MAVEN_BASE_URL, forge_version, terminal
    );
    let bytes = download_bytes_from_url(&url).await?;

    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)?;
    let version_file = archive.by_name("version.json")?;

    let version_bytes: Vec<u8> = version_file
        .bytes()
        .filter_map(|byte| match byte {
            Ok(b) => Some(b),
            Err(_) => None,
        })
        .collect();
    Ok(serde_json::from_slice(&version_bytes)?)
}

#[test]
pub fn test_download_forge_hashes() {
    let forge_version = "1.19.3-44.1.8";

    tauri::async_runtime::block_on(async move {
        let x = download_forge_hashes(forge_version).await;
        assert!(x.is_ok());
        println!("Hashes: {:#?}", x.unwrap());
    });
}

#[test]
pub fn test_download_forge_version() {
    let forge_version = "1.19.3-44.1.8";

    tauri::async_runtime::block_on(async move {
        let x = download_forge_version(forge_version, None).await;
        assert!(x.is_ok());
        println!("Result: {:#?}", x.unwrap());
    });
}
