use std::{path::{PathBuf, Path}, sync::Arc, collections::HashMap};

use log::info;
use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};
use tauri::{async_runtime::Mutex, AppHandle, Wry, State, Manager};

const VANILLA_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Deserialize)]
struct VanillaLatest {
    release: String,
    snapshot: String,
}

#[derive(Debug, Deserialize)]
struct VanillaVersion {
    id: String,
    #[serde(rename = "type")]
    version_type: String,
    url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
    sha1: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32
}

#[derive(Debug, Deserialize)]
struct VanillaManifest {
    latest: VanillaLatest,
    versions: Vec<VanillaVersion>,
}

type ManifestResult<T> = Result<T, ManifestError>;

enum Modloader {
    Vanilla,
    Forge,
    Fabric,
}
pub enum ManifestError {
    HttpError(reqwest::Error),
}

impl Serialize for ManifestError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer {
        match &self {
            ManifestError::HttpError(error) =>  serializer.serialize_str(&error.to_string()),
        }
    }
}

impl From<reqwest::Error> for ManifestError {
    fn from(e: reqwest::Error) -> Self {
        ManifestError::HttpError(e)
    }
}

async fn obtain_vanilla_manifest() ->  ManifestResult<VanillaManifest> {
    let client = reqwest::Client::new();
    let response = client
    .get(VANILLA_MANIFEST_URL)
    .send()
    .await?;

    let manifest = response.json::<VanillaManifest>().await?;
    Ok(manifest)
}

// async fn validate_hash(version: &VanillaVersion) -> ManifestResult<()> {
//     let mut hasher = Sha1::new();
//     hasher.update(response.text().await?);
//     let result = hasher.finalize();
//     println!("Hasher Result: {:#?}", result);

//     Ok(())
// }


#[tauri::command]
pub async fn obtain_manifests(app_handle: AppHandle<Wry>) -> ManifestResult<HashMap<String, Vec<String>>> {
    let resource_state: State<ResourceState> = app_handle.try_state().expect("`ResourceState` should already be managed.");
    let mut resource_manager = resource_state.0.lock().await;

    let versions = resource_manager.obtain_manifests().await?;
    Ok(versions)
}


pub struct ResourceState(Arc<Mutex<ResourceManager>>);

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

impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            vanilla_manifest: None,
        }
    }

    pub async fn obtain_manifests(&mut self) -> ManifestResult<HashMap<String, Vec<String>>> { 
        self.obtain_vanilla_manifest().await?;
        // TODO: Validate hash.

        let unique_versions = self.get_unique_versions();
        Ok(unique_versions)
    }

    // TODO: Return a hashmap of type to versions. Ex: <"Forge", 1.19.2>, <"Vanilla", 1.19.2>
    fn get_unique_versions(&self) -> HashMap<String, Vec<String>> {
        let mut versions: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(vanilla) = &self.vanilla_manifest {
            let mut vanilla_versions: Vec<String> = Vec::new();
            for manifest in vanilla.versions.iter() {
                vanilla_versions.push(manifest.id.clone());
            }
            versions.insert("vanilla".into(), vanilla_versions);
        }
        versions
    }

    async fn obtain_vanilla_manifest(&mut self) -> ManifestResult<()> {
        if let None = self.vanilla_manifest {
            self.vanilla_manifest = Some(obtain_vanilla_manifest().await?);
        }
        Ok(())
    }

}