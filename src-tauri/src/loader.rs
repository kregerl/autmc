use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc, io::{self, Write, BufReader, Read}, fs::{File, self}, string::FromUtf8Error, time::Instant
};
use bytes::Bytes;
use crypto::{sha1::Sha1, digest::Digest};
use indexmap::IndexMap;
use log::{info, error};
use serde::{de::{Visitor, SeqAccess, Error}, Deserialize, Deserializer, Serialize};
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
    ResourceError(String),
    InvalidFileDownload(String)
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

            while let Some(elem) = seq.next_element::<String>()? {
                vec.push(elem);
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

#[derive(Debug)]
enum JarType {
    Client,
    Server
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

/// Download the bytes for a file at the specified `url`
async fn obtain_file_bytes(url: &str) -> ManifestResult<Bytes> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.bytes().await?)
}

/// Validates that the hash of `bytes` matches the `valid_hash`
fn validate_hash(bytes: &Bytes, valid_hash: &str) -> bool {
    let mut hasher = Sha1::new();
    hasher.input(bytes);
    let result = hasher.result_str();
    result == valid_hash
}

/// Validates that the `path` exists and that the hash of it matches `valid_hash`
fn validate_file_hash(path: &Path, valid_hash: &str) -> bool {
    if !path.exists() {
        return false;
    }
    let result = read_bytes_from_file(path);
    if let Ok(bytes) = result {
        let valid = validate_hash(&bytes, &valid_hash);
        info!("REMOVEME: Is file valid: {}", valid);
        valid
    } else {
        false
    }
}

fn read_bytes_from_file(path: &Path) -> ManifestResult<Bytes> {
    let mut file = File::open(&path)?;
    let metadata = file.metadata()?;
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer)?;
    Ok(Bytes::from(buffer))
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
    
    resource_manager.download_game_jar(JarType::Client, &version.downloads.client, &version.id).await?;

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
                // If there is a version json cached and its hash matches the manifest hash, load it.
                if validate_file_hash(&self.get_version_file_path(version_id), &manifest_version.sha1) {
                    info!("Loading vanilla version `{}` from disk.", version_id);
                    self.deserialize_cached_vanilla_version(version_id)
                } else {
                    info!("Requesting vanilla version from {}", &manifest_version.url);
                    let bytes = obtain_vanilla_version(&manifest_version.url).await?;
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
            Err(ManifestError::ResourceError("Trying to access vanilla manifest but it is not downloaded yet.".into()))
        }
    }

    async fn download_libraries(&self, libraries: &[Library]) -> ManifestResult<()> {
        info!("Downloading {} libraries...", libraries.len());
        if !self.libraries_dir.exists() {
            fs::create_dir(&self.libraries_dir)?;
        }
        let start = Instant::now();
        for library in libraries {
            let artifact = &library.downloads.artifact;
            let path = self.libraries_dir.join(&artifact.path);
            // Create all parent dirs if they dont already exist.
            let parent_dir = path.parent().unwrap();
            fs::create_dir_all(parent_dir)?;

            if path.exists() && validate_file_hash(&path, &artifact.metadata.sha1) {
                info!("Library {} already exists, skipping download.", library.name);
                continue;
            } else {
                info!("Downloading library {}", &library.name);
                // Write bytes into the jar file specified.
                let bytes = obtain_file_bytes(&artifact.metadata.url).await?;
                if !validate_hash(&bytes, &artifact.metadata.sha1) {
                    let err = format!("Error downloading {}, invalid hash.", &library.name);
                    error!("{}", err);
                    return Err(ManifestError::InvalidFileDownload(err));
                }
                let mut file = File::create(path)?;
                file.write_all(&bytes)?;
            }
        }
        info!("Successfully downloaded libraries in {}ms", start.elapsed().as_millis());
        Ok(())
    }

    /// Downloads a game jar (client or server) to ${app_dir}/versions/(client|server)/${version_id}.jar
    async fn download_game_jar(&self, jar_type: JarType, download: &DownloadMetadata, version_id: &str) -> ManifestResult<()> {
        let jar_str = match jar_type {
            JarType::Client => "client",
            JarType::Server => "server",
        };
        // Create all dirs in path to file location.
        let dir_path = &self.version_dir.join(&jar_str);
        fs::create_dir_all(dir_path)?;

        let path = dir_path.join(format!("{}.jar", version_id));
        // Check if the file exists and the hash matches the download's sha1.
        if !validate_file_hash(&path, &download.sha1) {
            info!("Downloading {} {} jar", version_id, jar_str);
            let bytes = obtain_file_bytes(&download.url).await?;
            if !validate_hash(&bytes, &download.sha1) {
                let err = format!("Error downloading {} {} jar, invalid hash.", version_id, jar_str); 
                error!("{}", err);
                return Err(ManifestError::InvalidFileDownload(err));
            }
            let mut file = File::create(path)?;
            file.write_all(&bytes)?;
        }
        Ok(())
    }

    /// Gets the path to a version json given a `version_id`
    fn get_version_file_path(&self, version_id: &str) -> PathBuf {
        self.version_dir.join(format!("{}.json", version_id))
    }

    /// Deserialize a cached vanilla version json from disk.
    fn deserialize_cached_vanilla_version(&self, version_id: &str) -> ManifestResult<VanillaVersion> {
        let path = self.version_dir.join(format!("{}.json", version_id));
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let version  = serde_json::from_reader::<BufReader<File>, VanillaVersion>(reader)?;
        Ok(version)
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
