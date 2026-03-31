use std::path::{Path, PathBuf};

use autmc_downloader::{DownloadResult, Downloadable, download_bytes_from_url, download_json_object_from_url};
use serde::Deserialize;

use crate::{LaunchArguments, ManifestResult, get_directory_separator, maven_to_vec};

#[derive(Debug, Deserialize)]
pub struct FabricLibrary {
    name: String,
    url: String,
}

#[derive(Debug)]
pub struct DownloadableFabricLibrary {
    name: String,
    url: String,
    hash: String,
}

impl Downloadable for DownloadableFabricLibrary {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> String {
        self.url.to_owned()
    }

    fn hash(&self) -> &str {
        &self.hash
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        let path = maven_to_vec(&self.name, None, None).join(get_directory_separator());
        base_dir.join(path)
    }
}

#[derive(Debug, Deserialize)]
pub struct FabricProfile {
    // id: String,
    // #[serde(rename = "inheritsFrom")]
    // inherits_from: String,
    // #[serde(rename = "releaseTime")]
    // release_time: String,
    // time: String,
    // #[serde(rename = "type")]
    // version_type: String,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    pub arguments: LaunchArguments,
    pub libraries: Vec<FabricLibrary>,
}
pub const FABRIC_BASE_URL: &str = "https://meta.fabricmc.net/v2";

pub async fn download_fabric_profile(
    minecraft_version: &str,
    fabric_version: &str,
) -> DownloadResult<FabricProfile> {
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        FABRIC_BASE_URL, minecraft_version, fabric_version
    );
    Ok(download_json_object_from_url::<FabricProfile>(&url).await?)
}

pub async fn obtain_fabric_library_hashes(
    libraries: &[FabricLibrary],
) -> ManifestResult<Vec<DownloadableFabricLibrary>> {
    let mut result = Vec::with_capacity(libraries.len());
    for library in libraries {
        let hash_url = format!(
            "{}{}",
            library.url,
            maven_to_vec(&library.name, None, Some(".jar.sha1")).join(get_directory_separator())
        );
        let bytes = download_bytes_from_url(&hash_url).await?;
        let hash = String::from_utf8(bytes.to_vec())?;
        result.push(DownloadableFabricLibrary {
            name: library.name.to_owned(),
            url: format!(
                "{}{}",
                library.url,
                maven_to_vec(&library.name, None, None).join(get_directory_separator())
            ),
            hash,
        });
    }

    Ok(result)
}