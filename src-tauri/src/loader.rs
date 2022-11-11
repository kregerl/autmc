use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc, io::{self, Write, BufReader}, fs::{File, self}, string::FromUtf8Error,
};
use bytes::Bytes;
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

fn as_version_map<'de, D>(deserializer: D) -> Result<IndexMap<String, VanillaManifestVersion>, D::Error>
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
    SerializationFilesystemError(io::Error),
    Utf8DeserializationError(FromUtf8Error),
    JsonSerializationError(serde_json::Error),
    VersionRetrievalError(String),
    ResourceError(String)
}

impl Serialize for ManifestError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self {
            ManifestError::HttpError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::SerializationFilesystemError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::Utf8DeserializationError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::JsonSerializationError(error) => serializer.serialize_str(&error.to_string()),
            ManifestError::VersionRetrievalError(error) => serializer.serialize_str(&error),
            ManifestError::ResourceError(error) => serializer.serialize_str(&error),
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

#[derive(Debug, Deserialize, Serialize)]
enum RuleType {
    #[serde(rename = "features")]
    Features(HashMap<String, bool>),
    #[serde(rename = "os")]
    OperatingSystem(HashMap<String, String>),
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    action: String,
    #[serde(flatten)]
    rule_type: RuleType,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
struct LaunchArguments {
    game: Vec<Argument>,
    jvm: Vec<Argument>
}

#[derive(Debug, Deserialize, Serialize)]
struct DownloadMetadata {
    sha1: String,
    size: u32,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct AssetIndex {
    id: String,
    #[serde(flatten)]
    metadata: DownloadMetadata,
    #[serde(rename = "totalSize")]
    total_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct GameDownloads {
    client: DownloadMetadata,
    client_mappings: DownloadMetadata,
    server: DownloadMetadata,
    server_mappings: DownloadMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
struct JavaVersion {
    component: String,
    #[serde(rename = "majorVersion")]
    major_version: u32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Artifact {
    path: String,
    #[serde(flatten)]
    metadata: DownloadMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
struct LibraryDownloads {
    artifact: Artifact,
}

#[derive(Debug, Deserialize, Serialize)]
struct Library {
    downloads: LibraryDownloads,
    name: String,
    rules: Option<Vec<Rule>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClientLoggerFile {
    id: String,
    #[serde(flatten)]
    metadata: DownloadMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClientLogger {
    argument: String,
    file: ClientLoggerFile,
    #[serde(rename = "type")]
    logger_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Logging {
    client: ClientLogger,
}

#[derive(Debug, Deserialize, Serialize)]
/// The launch arguments and metadata for a given vanilla version.
// REVIEW: I believe this response is different for older versions of the game. versions < 1.13
struct VanillaVersion {
    arguments: LaunchArguments,
    #[serde(rename = "assetIndex")]
    asset_index: AssetIndex,
    assets: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32,
    downloads: GameDownloads,
    id: String,
    #[serde(rename = "javaVersion")]
    java_version: JavaVersion,
    libraries: Vec<Library>,
    logging: Logging,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[serde(rename = "minimumLauncherVersion")]
    min_launcher_version: u32,
    #[serde(rename = "releaseTime")]
    release_time: String,
    time: String,
    #[serde(rename = "type")]
    version_type: String,
}

// async fn obtain_vanilla_manifest() -> ManifestResult<VanillaManifest> {
//     let client = reqwest::Client::new();
//     let response = client.get(VANILLA_MANIFEST_URL).send().await?;

//     let manifest = response.json::<VanillaManifest>().await?;
//     Ok(manifest)
// }

async fn obtain_vanilla_version(url: &str) -> ManifestResult<Bytes> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.bytes().await?)
}

async fn obtain_library_jar_bytes(url: &str) -> ManifestResult<Bytes> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.bytes().await?)
}

fn validate_hash(bytes: &Bytes, valid_hash: &str) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    println!("Hasher Result: {:#?}", result);
    false
}

#[tauri::command]
pub async fn obtain_manifests(show_snapshots: bool, app_handle: AppHandle<Wry>) -> ManifestResult<Vec<String>> {
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

    let version = resource_manager.download_vanilla_version(&selected).await?;

    resource_manager.download_libraries(&version.libraries).await?;
    info!("Completed. Got version: {:#?}", version);
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
    version_dir: PathBuf,
    libraries_dir: PathBuf,
    vanilla_manifest: Option<VanillaManifest>,
    // TODO: Forge and Fabric manifests.
}

// TODO: Validate hashes when a version is selected.
impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            version_dir: app_dir.join("versions"),
            libraries_dir: app_dir.join("libraries"),
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

    /// Gets a list of all vanilla versions, including snapshots and old_beta if show_snapshots is true.
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

    async fn download_vanilla_version(&self, version_id: &str) -> ManifestResult<VanillaVersion> {
        if let Some(manifest) = &self.vanilla_manifest {
            if let Some(manifest_version) = manifest.versions.get(version_id) {
                if self.has_version_cached(version_id) {
                    // TODO: Check that the hash of the downloaded file is the same as  the hash in the manifest.
                    info!("Loading vanilla version `{}` from disk.", version_id);
                    self.deserialize_cached_vanilla_version(version_id)
                } else {
                    info!("Requesting vanilla version from {}", &manifest_version.url);
                    let bytes = obtain_vanilla_version(&manifest_version.url).await?;

                    info!("REMOVEME: Serializing vanilla version {}", version_id);
                    self.serialize_version(&version_id, &bytes)?;

                    info!("REMOVEME: Reading vanilla version struct from string");
                    let byte_str = String::from_utf8(bytes.to_vec())?;
                    let vanilla_version = serde_json::from_str::<VanillaVersion>(&byte_str)?;
                    Ok(vanilla_version)
                }
            } else {
                return Err(ManifestError::VersionRetrievalError(format!(
                    "Cannot find version with id: {}",
                    version_id
                )));
            }
        } else {
            Err(ManifestError::ResourceError("Trying to access vanilla manifest but it is not downloaded yet.".into()))
        }
    }

    async fn download_libraries(&self, libraries: &[Library]) -> ManifestResult<()> {
        if !self.libraries_dir.exists() {
            fs::create_dir(&self.libraries_dir)?;
        }
        for library in libraries {
            let artifact = &library.downloads.artifact;
            let path = self.libraries_dir.join(&artifact.path);
            // Create all parent dirs if they dont already exist.
            let parent_dir = path.parent().unwrap();
            fs::create_dir_all(parent_dir)?;
            
            // Write bytes into the jar file specified.
            let mut file = File::open(path)?;
            let bytes = obtain_library_jar_bytes(&artifact.metadata.url).await?;
            file.write_all(&bytes)?;
            validate_hash(&bytes, &artifact.metadata.sha1);
        }
        Ok(())
    }

    /// Checks to see if ${app_dir}/versions/${version_id}.json exists.
    fn has_version_cached(&self, version_id: &str) -> bool {
        let version_file_path = self.version_dir.join(format!("{}.json", version_id));
        version_file_path.exists()
    }

    /// Deserialize a cached vanilla version json from disk.
    fn deserialize_cached_vanilla_version(&self, version_id: &str) -> ManifestResult<VanillaVersion> {
        let path = self.version_dir.join(format!("{}.json", version_id));
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(serde_json::from_reader::<BufReader<File>, VanillaVersion>(reader)?)
    }

    /// Seralize a vanilla version from bytes to disk.
    fn serialize_version(&self, version_id: &str, bytes: &Bytes) -> Result<(), io::Error> {
        info!("REMOVEME: Serializing version json.");
        if !&self.version_dir.exists() {
            fs::create_dir(&self.version_dir)?;
        }
        let path = &self.version_dir.join(format!("{}.json", version_id));
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    }

}
