use std::{
    collections::{HashMap, btree_map::Entry},
    io::{Cursor, Read}, path::Path,
};

use serde::Deserialize;
use tauri::api::version;
use zip::read::ZipFile;

use crate::{
    consts::{FORGE_FILES_BASE_URL, FORGE_MAVEN_BASE_URL},
    state::resource_manager::ManifestResult,
    web_services::{downloader::{download_bytes_from_url, download_json_object, DownloadResult}, resources::get_classpath_separator},
};

use super::vanilla::{LaunchArguments, Library};

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
    pub arguments: LaunchArguments,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Deserialize)]
pub struct ForgeData {
    client: String,
    server: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgeProcessor {
    sides: Option<Vec<String>>,
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
    outputs: Option<HashMap<String, String>>
}

#[derive(Debug, Deserialize)]
pub struct ForgeInstall {
    spec: u32,
    profile: String,
    version: String,
    // path: Option<String>,
    minecraft: String,
    #[serde(rename = "serverJarPath")]
    server_jar_path: String,
    data: HashMap<String, ForgeData>,
    pub processors: Vec<ForgeProcessor>,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Deserialize)]
pub struct ForgeProfile {
    pub version: ForgeVersion,
    pub profile: ForgeInstall,
}

pub async fn download_forge_hashes(forge_version: &str) -> DownloadResult<ForgeHashes> {
    let url = format!("{}/{}/meta.json", FORGE_FILES_BASE_URL, forge_version);
    Ok(download_json_object::<ForgeHashes>(&url).await?)
}

fn bytes_from_zip_file(file: ZipFile) -> Vec<u8> {
    file
    .bytes()
    .filter_map(|byte| match byte {
        Ok(b) => Some(b),
        Err(_) => None,
    })
    .collect()
}

// TODO: Validate jar hash
pub async fn download_forge_version(
    forge_version: &str,
    valid_hash: Option<ForgeFileHash>,
) -> ManifestResult<ForgeProfile> {
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

    let version_bytes = bytes_from_zip_file(version_file);
        
    let install_profile_file = archive.by_name("install_profile.json")?;
    let install_profile_bytes = bytes_from_zip_file(install_profile_file);

    Ok(ForgeProfile {
        profile: serde_json::from_slice(&install_profile_bytes)?,
        version: serde_json::from_slice(&version_bytes)?
    })
}

// TODO: Make a single maven to vec that lets the caller use .join("/")
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

fn patch_forge(processors: &[ForgeProcessor], libraries_dir: &Path) {
    for processor in processors {
        let jar_path = libraries_dir.join(maven_to_fabric_endpoint(&processor.jar, None));
        let endpoints: Vec<_> = processor.classpath.iter().map(|classpath| maven_to_fabric_endpoint(classpath,None)).collect();
        let x: Vec<_> = endpoints.into_iter().map(|endpoint| libraries_dir.join(endpoint).into_os_string().into_string().unwrap()).collect();
        
        
        println!("Jar: {:#?}", jar_path);
        println!("Endpoints: {:#?}", x.join(&get_classpath_separator()));
        println!();
    }
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
        println!("test_download_forge_version: {:#?}", &x);
        assert!(x.is_ok());
        // println!("Result: {:#?}", x.unwrap());
        let y = patch_forge(&x.unwrap().profile.processors, Path::new("/home/loucas/.config/com.autm.launcher/libraries"));

    });
}
