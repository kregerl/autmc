use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    consts::FABRIC_BASE_URL,
    state::resource_manager::ManifestResult,
    web_services::downloader::{download_bytes_from_url, download_json_object, Downloadable},
};

use super::vanilla::{LaunchArguments, LaunchArguments113};

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    separator: String,
    build: i32,
    maven: String,
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderManifest(pub Vec<FabricLoaderVersion>);

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
        let mut path = maven_to_fabric_endpoint(&self.name, None);
        #[cfg(target_family = "windows")]
        {
            path = path.replace("/", &get_directory_separator());
        }
        base_dir.join(path)
    }
}

#[allow(dead_code)]
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
    pub main_class: String,
    pub arguments: LaunchArguments,
    pub libraries: Vec<FabricLibrary>,
}

pub async fn download_fabric_profile(
    minecraft_version: &str,
    fabric_version: &str,
) -> ManifestResult<FabricProfile> {
    let url = format!(
        "{}/versions/loader/{}/{}/profile/json",
        FABRIC_BASE_URL, minecraft_version, fabric_version
    );
    Ok(download_json_object::<FabricProfile>(&url).await?)
}

pub async fn obtain_fabric_library_hashes(
    libraries: &[FabricLibrary],
) -> ManifestResult<Vec<DownloadableFabricLibrary>> {
    let mut result = Vec::with_capacity(libraries.len());
    for library in libraries {
        let hash_url = format!(
            "{}{}",
            library.url,
            maven_to_fabric_endpoint(&library.name, Some(".sha1"))
        );
        println!("Hash Url: {}", hash_url);
        let bytes = download_bytes_from_url(&hash_url).await?;
        let hash = String::from_utf8(bytes.to_vec())?;
        result.push(DownloadableFabricLibrary {
            name: library.name.to_owned(),
            url: format!(
                "{}{}",
                library.url,
                maven_to_fabric_endpoint(&library.name, None)
            ),
            hash,
        });
    }

    Ok(result)
}

fn maven_to_fabric_endpoint(maven_artifact: &str, force_extension: Option<&str>) -> String {
    let splits: Vec<&str> = maven_artifact.split(":").collect();
    let file_name_ending = if splits.get(3).is_some() {
        format!("{}-{}", splits[2], splits[3])
    } else {
        splits[2].into()
    };

    let full_file_name = if file_name_ending.contains("@") {
        file_name_ending.replace("@", ".")
    } else {
        format!(
            "{}.jar{}",
            file_name_ending,
            if let Some(ext) = force_extension {
                ext
            } else {
                ""
            }
        )
    };

    let mut result = Vec::new();
    result.append(&mut splits[0].split(".").collect::<Vec<&str>>());
    result.push(splits[1]);
    result.push(splits[2].split("@").collect::<Vec<&str>>()[0]);
    let final_name = format!("{}-{}", splits[1], full_file_name);
    result.push(&final_name);

    result.join("/")
}

#[test]
fn test_maven_to_fabric() {
    let lib = FabricLibrary {
        name: "org.ow2.asm:asm-commons:9.2".into(),
        url: "https://maven.fabricmc.net/".into(),
    };

    let result = format!("{}{}", lib.url, maven_to_fabric_endpoint(&lib.name, None));
    assert!(result == "https://maven.fabricmc.net/org/ow2/asm/asm-commons/9.2/asm-commons-9.2.jar");
}

#[test]
fn test_fabric_profile() {
    let game_version = "1.19.3";
    let fabric_version = "0.14.3";

    tauri::async_runtime::block_on(async move {
        let x = download_fabric_profile(game_version, fabric_version).await;
        assert!(x.is_ok());
        let hashes = obtain_fabric_library_hashes(&x.unwrap().libraries).await;
        assert!(hashes.is_ok());
    });
}
