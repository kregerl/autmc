use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Write},
    path::{Path, PathBuf},
    string::FromUtf8Error,
    sync::Arc,
};

use bytes::Bytes;
use log::info;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use zip::result::ZipError;

use crate::{
    commands::VersionEntry,
    consts::{FABRIC_BASE_URL, FORGE_MANIFEST_URL, VANILLA_MANIFEST_URL},
    web_services::{
        downloader::{
            download_bytes_from_url, validate_file_hash, validate_hash_sha1, DownloadError,
        },
        manifest::{
            fabric::FabricLoaderManifest,
            forge::ForgeManifest,
            vanilla::{VanillaManifest, VanillaManifestVersion, VanillaVersion},
        },
    },
};

use super::{InnerState, ManagerFromAppHandle};

pub type ManifestResult<T> = Result<T, ManifestError>;

#[derive(Debug)]
pub enum ManifestError {
    HttpError(reqwest::Error),
    SerializationFilesystemError(io::Error),
    Utf8DeserializationError(FromUtf8Error),
    JsonSerializationError(serde_json::Error),
    VersionRetrievalError(String),
    ResourceError(String),
    MismatchedFileHash(String),
    FileExtractionError(ZipError),
}

impl Serialize for ManifestError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            ManifestError::HttpError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::SerializationFilesystemError(error) => {
                serializer.serialize_str(&error.to_string())
            }
            ManifestError::Utf8DeserializationError(error) => {
                serializer.serialize_str(&error.to_string())
            }
            ManifestError::JsonSerializationError(error) => {
                serializer.serialize_str(&error.to_string())
            }
            ManifestError::VersionRetrievalError(error) => serializer.serialize_str(error),
            ManifestError::ResourceError(error) => serializer.serialize_str(error),
            ManifestError::MismatchedFileHash(error) => serializer.serialize_str(error),
            ManifestError::FileExtractionError(error) => {
                serializer.serialize_str(&error.to_string())
            }
        }
    }
}

impl From<reqwest::Error> for ManifestError {
    fn from(e: reqwest::Error) -> Self {
        ManifestError::HttpError(e)
    }
}

impl From<io::Error> for ManifestError {
    fn from(error: io::Error) -> Self {
        ManifestError::SerializationFilesystemError(error)
    }
}

impl From<FromUtf8Error> for ManifestError {
    fn from(error: FromUtf8Error) -> Self {
        ManifestError::Utf8DeserializationError(error)
    }
}

impl From<serde_json::Error> for ManifestError {
    fn from(error: serde_json::Error) -> Self {
        ManifestError::JsonSerializationError(error)
    }
}

impl From<DownloadError> for ManifestError {
    fn from(error: DownloadError) -> Self {
        match error {
            DownloadError::Request(e) => ManifestError::HttpError(e),
            DownloadError::FileWrite(e) => ManifestError::SerializationFilesystemError(e),
            DownloadError::InvalidFileHash(e) => ManifestError::MismatchedFileHash(e),
        }
    }
}

impl From<ZipError> for ManifestError {
    fn from(error: ZipError) -> Self {
        ManifestError::FileExtractionError(error)
    }
}

pub struct ResourceState(pub Arc<Mutex<ResourceManager>>);

impl InnerState<Arc<Mutex<ResourceManager>>> for ResourceState {
    fn inner_state(&self) -> Arc<Mutex<ResourceManager>> {
        self.0.clone()
    }
}

impl ManagerFromAppHandle for ResourceManager {
    type State = ResourceState;
}

impl ResourceState {
    pub fn new(app_dir: &Path) -> Self {
        Self(Arc::new(Mutex::new(ResourceManager::new(app_dir))))
    }
}

#[derive(Debug)]
pub struct ResourceManager {
    app_dir: PathBuf,
    // FIXME: On instantiation of the resource manager, get all manifests so theres no options.
    vanilla_manifest: Option<VanillaManifest>,
    forge_manifest: Option<ForgeManifest>,
    fabric_manifest: Option<FabricLoaderManifest>,
}

impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            vanilla_manifest: None,
            forge_manifest: None,
            fabric_manifest: None,
        }
    }

    /// Returns the version directory at ${app_dir}/versions
    pub fn version_dir(&self) -> PathBuf {
        self.app_dir.join("versions")
    }

    /// Returns the libraries directory at ${app_dir}/libraries
    pub fn libraries_dir(&self) -> PathBuf {
        self.app_dir.join("libraries")
    }

    /// Returns the assets directory at ${app_dir}/assets
    pub fn assets_dir(&self) -> PathBuf {
        self.app_dir.join("assets")
    }

    /// Returns the objects directory at ${app_dir}/assets/objects
    pub fn asset_objects_dir(&self) -> PathBuf {
        self.assets_dir().join("objects")
    }

    /// Returns the java directory at ${app_dir}/java
    pub fn java_dir(&self) -> PathBuf {
        self.app_dir.join("java")
    }

    /// Returns the java directory at ${app_dir}/instances
    pub fn instances_dir(&self) -> PathBuf {
        self.app_dir.join("instances")
    }

    async fn download_fabric_manifest(&mut self) -> reqwest::Result<()> {
        info!("Downloading fabric manifest");
        let client = reqwest::Client::new();
        let fabric_manifest_url = format!("{}/{}", FABRIC_BASE_URL, "versions/loader");
        let fabric_response = client.get(fabric_manifest_url).send().await?;
        let fabric_manifest = fabric_response.json::<FabricLoaderManifest>().await?;
        self.fabric_manifest = Some(fabric_manifest);
        Ok(())
    }

    async fn download_forge_manifest(&mut self) -> reqwest::Result<()> {
        info!("Downloading forge manifest");
        let client = reqwest::Client::new();
        let forge_response = client.get(FORGE_MANIFEST_URL).send().await?;
        let forge_manifest = forge_response.json::<ForgeManifest>().await?;
        self.forge_manifest = Some(forge_manifest);
        Ok(())
    }

    async fn download_vanilla_manifest(&mut self) -> reqwest::Result<()> {
        info!("Downloading vanilla manifest");
        let client = reqwest::Client::new();
        let vanilla_response = client.get(VANILLA_MANIFEST_URL).send().await?;
        let vanilla_manifest = vanilla_response.json::<VanillaManifest>().await?;
        self.vanilla_manifest = Some(vanilla_manifest);
        Ok(())
    }

    /// Gets a list of all vanilla versions
    pub async fn get_vanilla_version_list(&mut self) -> reqwest::Result<Vec<VersionEntry>> {
        let mut result: Vec<VersionEntry> = Vec::new();
        if self.vanilla_manifest.is_none() {
            self.download_vanilla_manifest().await?;
        }
        if let Some(manifest) = &self.vanilla_manifest {
            for (version, version_info) in &manifest.versions {
                result.push(VersionEntry::new(version, version_info));
            }
        }
        Ok(result)
    }

    pub async fn get_fabric_version_list(&mut self) -> reqwest::Result<Vec<String>> {
        let mut result = Vec::new();
        if self.fabric_manifest.is_none() {
            self.download_fabric_manifest().await?;
        }
        if let Some(manifest) = &self.fabric_manifest {
            let FabricLoaderManifest(vec) = manifest;
            for entry in vec {
                result.push(entry.version.to_owned());
            }
        }
        Ok(result)
    }

    pub async fn get_forge_version_list(
        &mut self,
    ) -> reqwest::Result<HashMap<String, Vec<String>>> {
        if self.forge_manifest.is_none() {
            self.download_forge_manifest().await?;
        }
        Ok(if let Some(manifest) = &self.forge_manifest {
            manifest.0.to_owned()
        } else {
            HashMap::new()
        })
    }

    /// Get the vanilla manifest for a given mc_version. Returns None if mc_version is invalid.
    pub fn get_vanilla_manifest_from_version(
        &self,
        mc_version: &str,
    ) -> Option<&VanillaManifestVersion> {
        if let Some(manifest) = &self.vanilla_manifest {
            manifest.versions.get(mc_version)
        } else {
            None
        }
    }

    pub async fn download_vanilla_version(
        &self,
        version_id: &str,
    ) -> ManifestResult<VanillaVersion> {
        if let Some(manifest) = &self.vanilla_manifest {
            if let Some(manifest_version) = manifest.versions.get(version_id) {
                // If there is a version json cached and its hash matches the manifest hash, load it.
                if validate_file_hash(
                    &self.get_version_file_path(version_id),
                    &manifest_version.sha1,
                ) {
                    info!("Loading vanilla version `{}` from disk.", version_id);
                    self.deserialize_cached_vanilla_version(version_id)
                } else {
                    info!("Requesting vanilla version from {}", &manifest_version.url);
                    let bytes = download_bytes_from_url(&manifest_version.url).await?;
                    validate_hash_sha1(&bytes, "");

                    self.serialize_version(version_id, &bytes)?;

                    let vanilla_version =
                        serde_json::from_slice::<VanillaVersion>(&bytes.to_vec())?;
                    info!("Finished downloading version `{}`", version_id);
                    Ok(vanilla_version)
                }
            } else {
                Err(ManifestError::VersionRetrievalError(format!(
                    "Cannot find version with id: {}",
                    version_id
                )))
            }
        } else {
            Err(ManifestError::ResourceError(
                "Trying to access vanilla manifest but it is not downloaded yet.".into(),
            ))
        }
    }

    /// Gets the path to a version json given a `version_id`
    fn get_version_file_path(&self, version_id: &str) -> PathBuf {
        self.version_dir().join(format!("{}.json", version_id))
    }

    /// Deserialize a cached vanilla version json from disk.
    fn deserialize_cached_vanilla_version(
        &self,
        version_id: &str,
    ) -> ManifestResult<VanillaVersion> {
        let path = self.version_dir().join(format!("{}.json", version_id));
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let version = serde_json::from_reader::<BufReader<File>, VanillaVersion>(reader)?;
        Ok(version)
    }

    /// Seralize a vanilla version from bytes to disk.
    fn serialize_version(&self, version_id: &str, bytes: &Bytes) -> Result<(), io::Error> {
        info!("REMOVEME: Serializing version json.");
        if !&self.version_dir().exists() {
            fs::create_dir(self.version_dir())?;
        }
        let dir_path = &self.version_dir().join(version_id);
        fs::create_dir_all(dir_path)?;

        let path = &dir_path.join(format!("{}.json", version_id));
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    }
}
