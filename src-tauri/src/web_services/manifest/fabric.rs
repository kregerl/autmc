use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{
    consts::FABRIC_BASE_URL,
    state::resource_manager::ManifestResult,
    web_services::downloader::{download_json_object, Downloadable},
};

use super::vanilla::LaunchArguments;

#[derive(Debug, Deserialize)]
pub struct FabricLoaderVersion {
    separator: String,
    build: i32,
    maven: String,
    pub version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
pub struct FabricLoaderManifest(pub Vec<FabricLoaderVersion>);

#[derive(Debug, Deserialize)]
pub struct FabricLibrary {
    name: String,
    url: String,
}

impl Downloadable for FabricLibrary {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> String {
        format!("{}{}", self.url, maven_to_fabric_endpoint(&self))
    }

    // FIXME: Get hash or make an option
    fn hash(&self) -> &str {
        ""
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(maven_to_fabric_endpoint(&self))
    }
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
    // TODO: Use custom deserializer with FabricLibrary so it can download the file hash too.
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

fn maven_to_fabric_endpoint(library: &FabricLibrary) -> String {
    let splits: Vec<&str> = library.name.split(":").collect();
    let file_name_ending = if splits.get(3).is_some() {
        format!("{}-{}", splits[2], splits[3])
    } else {
        splits[2].into()
    };

    let full_file_name = if file_name_ending.contains("@") {
        file_name_ending.replace("@", ".")
    } else {
        format!("{}.jar", file_name_ending)
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

    let result = format!("{}{}", lib.url, maven_to_fabric_endpoint(&lib));
    assert!(result == "https://maven.fabricmc.net/org/ow2/asm/asm-commons/9.2/asm-commons-9.2.jar");
}
