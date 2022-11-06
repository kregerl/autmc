use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use indexmap::IndexMap;
use log::info;
use serde::{de::{Visitor, SeqAccess, Error}, Deserialize, Deserializer, Serialize};
use sha1::{Digest, Sha1};
use tauri::{async_runtime::Mutex, AppHandle, Manager, State, Wry};

const VANILLA_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// We dont really care about the latest version since we preserve json order.
// #[derive(Debug, Deserialize)]
// struct VanillaLatest {
//     release: String,
//     snapshot: String,
// }

#[derive(Debug, Deserialize)]
/// The version metadata returned in the manifest request.
struct VanillaManifestVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    sha1: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32,
}

#[derive(Debug, Deserialize)]
/// Struct holding everything returned in the vanilla manifest json.
struct VanillaManifest {
    // latest: VanillaLatest,
    #[serde(deserialize_with = "as_version_map")]
    versions: IndexMap<String, VanillaManifestVersion>,
}

fn as_version_map<'de, D>(
    deserializer: D,
) -> Result<IndexMap<String, VanillaManifestVersion>, D::Error>
where
    D: Deserializer<'de>,
{
    let vanilla_versions: Vec<VanillaManifestVersion> = Deserialize::deserialize(deserializer)?;
    // IndexMap keeps insertion order. Important here when deserializing the json since the
    // vanilla manifest is in the correct order already.
    let mut map: IndexMap<String, VanillaManifestVersion> = IndexMap::new();
    for version in vanilla_versions {
        map.insert(version.id.clone(), version);
    }
    Ok(map)
}

type ManifestResult<T> = Result<T, ManifestError>;

#[derive(Debug)]
pub enum ManifestError {
    HttpError(reqwest::Error),
    VersionRetrievalError(String),
}

impl Serialize for ManifestError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            ManifestError::HttpError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::VersionRetrievalError(error) => serializer.serialize_str(&error),
        }
    }
}

impl From<reqwest::Error> for ManifestError {
    fn from(e: reqwest::Error) -> Self {
        ManifestError::HttpError(e)
    }
}

#[derive(Debug, Deserialize)]
enum RuleType {
    #[serde(rename = "features")]
    Features(HashMap<String, bool>),
    #[serde(rename = "os")]
    OperatingSystem(HashMap<String, String>),
}

#[derive(Debug, Deserialize)]
struct Rule {
    action: String,
    #[serde(flatten)]
    rule_type: RuleType,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Argument {
    Arg(String),
    ConditionalArg {
        rules: Vec<Rule>,
        #[serde(deserialize_with = "string_or_strings_as_vec")]
        value: Vec<String>,
    },
}

fn string_or_strings_as_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> Visitor<'de> for StringVisitor {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or array of strings.")
        }
        
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
            Ok(vec![v.into()])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
            let mut vec: Vec<String> = Vec::new();

            while let Some(elem) = seq.next_element::<&str>()? {
                vec.push(elem.into());
            }
            Ok(vec)
        }
    }
    deserializer.deserialize_any(StringVisitor)
}

#[derive(Debug, Deserialize)]
struct LaunchArguments {
    game: Vec<Argument>,
    jvm: Vec<Argument>
}

#[derive(Debug, Deserialize)]
struct AssetIndex {
    id: String,
    sha1: String,
    size: u32,
    #[serde(rename = "totalSize")]
    total_size: u32,
    url: String
}



#[derive(Debug, Deserialize)]
struct GameDownloads {

}

#[derive(Debug, Deserialize)]
/// The launch arguments and metadata for a given vanilla version.
// REVIEW: I believe this response is different for older versions of the game. versions < 1.12
struct VanillaVersion {
    arguments: LaunchArguments,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    assets: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32,
    downloads: GameDownloads
}

// async fn obtain_vanilla_manifest() -> ManifestResult<VanillaManifest> {
//     let client = reqwest::Client::new();
//     let response = client.get(VANILLA_MANIFEST_URL).send().await?;

//     let manifest = response.json::<VanillaManifest>().await?;
//     Ok(manifest)
// }

async fn obtain_vanilla_version(url: &str) -> ManifestResult<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    info!("Here: {}", &url);
    let vanilla_version = response.json::<VanillaVersion>().await?;
    info!("Got: {:#?}", vanilla_version);
    Ok(())
}

// async fn validate_hash(version: &VanillaVersion) -> ManifestResult<()> {
//     let mut hasher = Sha1::new();
//     hasher.update(response.text().await?);
//     let result = hasher.finalize();
//     println!("Hasher Result: {:#?}", result);

//     Ok(())
// }

#[tauri::command]
pub async fn obtain_manifests(
    show_snapshots: bool,
    app_handle: AppHandle<Wry>,
) -> ManifestResult<Vec<String>> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    let versions = resource_manager.get_vanilla_version_list(show_snapshots);
    Ok(versions)
}

#[tauri::command]
pub async fn obtain_version(selected: String, app_handle: AppHandle<Wry>) -> ManifestResult<()> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;

    resource_manager.download_vanilla_version(selected).await?;

    Ok(())
}

pub struct ResourceState(pub Arc<Mutex<ResourceManager>>);

impl ResourceState {
    pub fn new(app_dir: &PathBuf) -> Self {
        Self(Arc::new(Mutex::new(ResourceManager::new(app_dir))))
    }
}

#[derive(Debug)]
pub struct ResourceManager {
    app_dir: PathBuf,
    vanilla_manifest: Option<VanillaManifest>,
    // TODO: Forge and Fabric manifests.
}

// TODO: Validate hashes when a version is selected.
impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            vanilla_manifest: None,
        }
    }

    pub async fn download_manifests(&mut self) -> ManifestResult<()> {
        info!("Downloading manifests");
        let client = reqwest::Client::new();
        let response = client.get(VANILLA_MANIFEST_URL).send().await?;

        let manifest = response.json::<VanillaManifest>().await?;

        self.vanilla_manifest = Some(manifest);
        Ok(())
    }

    pub fn get_vanilla_version_list(&self, show_snapshots: bool) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        if let Some(manifest) = &self.vanilla_manifest {
            for (version, version_info) in &manifest.versions {
                if !show_snapshots && version_info.version_type == "release" {
                    result.push(version.clone());
                } else if show_snapshots {
                    // old_beta version types are considered snapshots in this context.
                    result.push(version.clone());
                }
            }
        }
        result
    }

    pub async fn download_vanilla_version(&self, version_id: String) -> ManifestResult<()> {
        if let Some(manifest) = &self.vanilla_manifest {
            info!("Have vanilla manifest");
            if let Some(version) = manifest.versions.get(&version_id) {
                info!("Downloading vanilla version");
                obtain_vanilla_version(&version.url).await?;
                info!("Done");
            } else {
                return Err(ManifestError::VersionRetrievalError(format!(
                    "Cannot find version with id: {}",
                    version_id
                )));
            }
        }
        Ok(())
    }
}
