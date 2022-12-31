use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, BufReader, Write, BufRead},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    string::FromUtf8Error,
    sync::Arc,
};

use bytes::Bytes;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use tauri::async_runtime::Mutex;

use crate::{
    commands::{VersionEntry, VersionFilter},
    consts::VANILLA_MANIFEST_URL,
    web_services::{
        downloader::{download_bytes_from_url, validate_file_hash, validate_hash},
        resources::{
            substitute_account_specific_arguments
        }, manifest::vanilla::{VanillaManifest, VanillaManifestVersion, VanillaVersion},
    },
};

use super::account_manager::Account;

pub type ManifestResult<T> = Result<T, ManifestError>;

#[derive(Debug)]
pub enum ManifestError {
    HttpError(reqwest::Error),
    SerializationFilesystemError(io::Error),
    Utf8DeserializationError(FromUtf8Error),
    JsonSerializationError(serde_json::Error),
    VersionRetrievalError(String),
    ResourceError(String),
    InvalidFileDownload(String),
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
            ManifestError::VersionRetrievalError(error) => serializer.serialize_str(&error),
            ManifestError::ResourceError(error) => serializer.serialize_str(&error),
            ManifestError::InvalidFileDownload(error) => serializer.serialize_str(&error),
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

#[derive(Serialize, Deserialize, Debug)]
pub struct InstanceConfiguration {
    pub instance_name: String,
    pub jvm_path: PathBuf,
    pub arguments: Vec<String>,
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
    instance_map: HashMap<String, InstanceConfiguration>,
    // TODO: Forge and Fabric manifests.
}

impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            vanilla_manifest: None,
            instance_map: HashMap::new(),
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

    pub async fn download_manifests(&mut self) -> ManifestResult<()> {
        info!("Downloading manifests");
        let client = reqwest::Client::new();
        let response = client.get(VANILLA_MANIFEST_URL).send().await?;

        let manifest = response.json::<VanillaManifest>().await?;

        self.vanilla_manifest = Some(manifest);
        Ok(())
    }

    /// Gets a list of all vanilla versions
    pub fn get_vanilla_version_list(&self, filters: &[VersionFilter]) -> Vec<VersionEntry> {
        let mut result: Vec<VersionEntry> = Vec::new();
        if let Some(manifest) = &self.vanilla_manifest {
            for (version, version_info) in &manifest.versions {
                for filter in filters {
                    if filter.checked && version_info.version_type == filter.id {
                        result.push(VersionEntry::new(version, version_info));
                    }
                }
            }
        }
        result
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

    /// Add the config.json to an instance folder. Used to relaunch the instance again.
    pub fn add_instance(&self, config: InstanceConfiguration) -> Result<(), io::Error> {
        let path = self
            .instances_dir()
            .join(&config.instance_name)
            .join("config.json");
        let mut file = File::create(path)?;
        let json = serde_json::to_string(&config)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn deserialize_instances(&mut self) {
        let paths = fs::read_dir(self.instances_dir());
        if let Err(e) = paths {
            error!("Error loading instances from disk: {}", e);
            return;
        }
        for path in paths.unwrap().filter_map(|path| path.ok()) {
            let instance_path = path.path().join("config.json");
            let file = File::open(&instance_path);
            if let Err(e) = file {
                warn!("Error with instance at {}: {}", instance_path.display(), e);
                continue;
            }
            let reader = BufReader::new(file.unwrap());
            let instance =
                serde_json::from_reader::<BufReader<File>, InstanceConfiguration>(reader);
            if let Err(e) = instance {
                warn!(
                    "Error loading `config.json` for instance at {}: {}",
                    instance_path.display(),
                    e
                );
                continue;
            }
            let conf = instance.unwrap();
            self.instance_map.insert(conf.instance_name.clone(), conf);
        }
    }

    pub fn get_instance_names(&self) -> Vec<String> {
        self.instance_map
            .iter()
            .map(|(instance_name, _)| instance_name.into())
            .collect()
    }

    // IDEA: Move to an `InstanceManager`?
    pub fn launch_instance(&self, instance_name: &str, active_account: &Account) {
        debug!("Instance Name: {}", instance_name);
        let instance_config = self.instance_map.get(instance_name);
        match instance_config {
            Some(instance) => {
                let working_dir = self.instances_dir().join(instance_name);
                let mut args: Vec<String> = Vec::new();
                for argument in &instance.arguments {
                    args.push(
                        match substitute_account_specific_arguments(argument, active_account) {
                            Some(arg) => arg,
                            None => argument.into(),
                        },
                    );
                }
                let mut command = Command::new(&instance.jvm_path);
                command.current_dir(working_dir).args(args).stdout(Stdio::piped());
                debug!("Command: {:#?}", command);
                let child = command.spawn().expect("Could not spawn instance.");
                
                // let reader = BufReader::new(child.stdout.unwrap());
                // for line in reader.lines() {
                //     debug!("Mc Log: {:#?}", line);
                // }
            }
            None => error!("Unknown instance name: {}", instance_name),
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
                    validate_hash(&bytes, "");

                    info!("REMOVEME: Serializing vanilla version {}", version_id);
                    self.serialize_version(&version_id, &bytes)?;

                    info!("REMOVEME: Reading vanilla version struct from string");
                    let byte_str = String::from_utf8(bytes.to_vec())?;
                    let vanilla_version = serde_json::from_str::<VanillaVersion>(&byte_str)?;
                    info!("Finished downloading version `{}`", version_id);
                    Ok(vanilla_version)
                }
            } else {
                return Err(ManifestError::VersionRetrievalError(format!(
                    "Cannot find version with id: {}",
                    version_id
                )));
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
            fs::create_dir(&self.version_dir())?;
        }
        let dir_path = &self.version_dir().join(version_id);
        fs::create_dir_all(dir_path)?;

        let path = &dir_path.join(format!("{}.json", version_id));
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    }
}
