use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc, io::{self, Write, BufReader, Read}, fs::{File, self}, string::FromUtf8Error, time::Instant, env, mem::size_of, ffi::OsStr
};
use bytes::Bytes;
use chrono::format;
use crypto::{sha1::Sha1, digest::Digest};
use indexmap::IndexMap;
use log::{info, error, warn, debug};
use serde::{de::{Visitor, SeqAccess, Error, DeserializeOwned}, Deserialize, Deserializer, Serialize};
use tauri::{async_runtime::Mutex, AppHandle, Manager, State, Wry, api::version};

use crate::downloader::{Downloadable, download_all_callback, DownloadError};

const VANILLA_MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// The url to download assets from. Uses the hash as the endpoint: `...net/<first 2 hex letters of hash>/<whole hash>`
const VANILLA_ASSET_BASE_URL: &str = "http://resources.download.minecraft.net";
const JAVA_VERSION_MANIFEST: &str = "https://launchermeta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";

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
struct Asset {
    path: String,
    hash: String,
    size: u32
}

impl Downloadable for Asset {
    fn name(&self) -> &str {
        &self.path
    }

    fn url(&self) -> String {
        let first_two_chars = &self.hash.split_at(2);
        let url = format!("{}/{}/{}", VANILLA_ASSET_BASE_URL, &first_two_chars.0, &self.hash);
        url
    }

    fn hash(&self) -> &str {
        &self.hash
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.path)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct AssetObject {
    #[serde(deserialize_with = "to_asset_vec")]
    objects: Vec<Asset>
}

fn to_asset_vec<'de, D>(deserializer: D) -> Result<Vec<Asset>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct TmpAsset {
        hash: String,
        size: u32
    }

    let asset_map: HashMap<String, TmpAsset> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(asset_map.len());
    for (path, tmp_asset) in asset_map {
        result.push(Asset {
            path,
            hash: tmp_asset.hash,
            size: tmp_asset.size
        });
    }
    Ok(result)
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
pub struct Library {
    downloads: LibraryDownloads,
    name: String,
    rules: Option<Vec<Rule>>,
}

impl Downloadable for Library {
    fn name(&self) -> &str {
        &self.name
    }

    fn url(&self) -> String {
        self.downloads.artifact.metadata.url.to_owned()
    }

    fn hash(&self) -> &str {
        &self.downloads.artifact.metadata.sha1
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.downloads.artifact.path)
    }
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
// TODO: What about server logging?
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

#[derive(Debug, Deserialize)]
struct JavaRuntimeAvailability {
    group: u32,
    progress: u32
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeVesion {
    name: String,
    released: String,
}

#[derive(Debug, Deserialize)]
struct JavaRuntime {
    availability: JavaRuntimeAvailability,
    manifest: DownloadMetadata,
    version: JavaRuntimeVesion,
}

#[derive(Debug, Deserialize)]
struct JavaManifest {
    #[serde(rename = "java-runtime-alpha", deserialize_with = "deserialize_java_runtime")]
    java_runtime_alpha: Option<JavaRuntime>,
    #[serde(rename = "java-runtime-beta", deserialize_with = "deserialize_java_runtime")]
    java_runtime_beta: Option<JavaRuntime>,
    #[serde(rename = "java-runtime-gamma", deserialize_with = "deserialize_java_runtime")]
    java_runtime_gamma: Option<JavaRuntime>,
    #[serde(rename = "jre-legacy", deserialize_with = "deserialize_java_runtime")]
    jre_legacy: Option<JavaRuntime>,
    #[serde(rename = "minecraft-java-exe", deserialize_with = "deserialize_java_runtime")]
    minecraft_java_exe: Option<JavaRuntime>,
}

fn deserialize_java_runtime<'de, D>(deserializer: D) -> Result<Option<JavaRuntime>, D::Error>
where
    D: Deserializer<'de>,
{

    let runtimes: Vec<JavaRuntime> = Deserialize::deserialize(deserializer)?;
    if runtimes.len() > 1 {
        warn!("Got more java runtimes than expected. Expected 1 but got {}", runtimes.len());
        Ok(None)
    } else {
        // We know we have atleast one element
        Ok(runtimes.into_iter().nth(0))
    }
}

#[cfg(test)]
pub async fn download_java_version_manifest() -> ManifestResult<()> {
    let client = reqwest::Client::new();
    let response = client.get(JAVA_VERSION_MANIFEST).send().await?;
    let java_version_manifest: HashMap<String, JavaManifest> = response.json().await?;
    println!("{:#?}", java_version_manifest);
    Ok(())
}

#[cfg(test)]
pub async fn download_java_runtime_manifest(url: &str) -> ManifestResult<()> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let java_runtime_manifest: JavaRuntimeManifest = response.json().await?;
    println!("{:#?}", java_runtime_manifest);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeDownload {
    lzma: Option<DownloadMetadata>,
    raw: DownloadMetadata,
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeFile {
    path: String,
    downloads: JavaRuntimeDownload,
    executable: bool,
}

impl Downloadable for JavaRuntimeFile {
    fn name(&self) -> &str {
        &self.path
    }

    // TODO: Would be better to use lzma download instead.
    fn url(&self) -> String {
        self.downloads.raw.url.to_owned()
    }

    fn hash(&self) -> &str {
        &self.downloads.raw.sha1
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.path)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum JavaRuntimeType {
    #[serde(rename = "file")]
    File(JavaRuntimeFile),
    #[serde(rename = "directory")]
    Directory(String),
    #[serde(rename = "link")]
    Link {
        path: String,
        target: String
    }
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeManifest {
    #[serde(deserialize_with = "to_java_runtime_vec")]
    files: Vec<JavaRuntimeType>,
}

fn to_java_runtime_vec<'de, D>(deserializer: D) -> Result<Vec<JavaRuntimeType>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    #[serde(tag = "type")]
    enum TmpJavaRuntimeType {
        #[serde(rename = "file")]
        File {
            downloads: JavaRuntimeDownload,
            executable: bool
        },
        #[serde(rename = "directory")]
        Directory,
        #[serde(rename = "link")]
        Link {
            target: String
        }
    }
    let jrt_map: HashMap<String, TmpJavaRuntimeType> = Deserialize::deserialize(deserializer)?;
    println!("HERE: {:#?}", jrt_map);
    let mut result = Vec::with_capacity(jrt_map.len());
    for (path, tmp_jrt) in jrt_map {
        result.push(match tmp_jrt {
            TmpJavaRuntimeType::File { downloads, executable } => JavaRuntimeType::File(JavaRuntimeFile {
                path,
                downloads,
                executable
            }),  
            TmpJavaRuntimeType::Directory => JavaRuntimeType::Directory(path),
            TmpJavaRuntimeType::Link { target } => JavaRuntimeType::Link{
                path,
                target
            },
        });
    }
    Ok(result)
}

async fn download_json_object<T>(url: &str) -> reqwest::Result<T> 
where T: DeserializeOwned
{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    Ok(response.json().await?)
}

/// Download the bytes for a file at the specified `url`
async fn download_bytes_from_url(url: &str) -> reqwest::Result<Bytes> {
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
//TODO: Use this when a `strict` setting is enabled.
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

/// Reads and returns bytes from the file specified in `path`
fn read_bytes_from_file(path: &Path) -> ManifestResult<Bytes> {
    let mut file = File::open(&path)?;
    let metadata = file.metadata()?;
    let mut buffer = vec![0; metadata.len() as usize];
    file.read(&mut buffer)?;
    Ok(Bytes::from(buffer))
}

/// Checks if a single rule matches every case. 
/// Returns true when an allow rule matches or a disallow rule does not match.
fn rule_matches(rule: &Rule) -> bool {
    match &rule.rule_type {
        RuleType::Features(_feature_rules) => todo!("Implement feature rules for arguments"),
        RuleType::OperatingSystem(os_rules) => {
            // Check if all the rules match the current system.
            let mut rule_matches = false;
            for (key, value) in os_rules {
                match key.as_str() {
                    "name" => {
                        let os_type = env::consts::OS;
                        if value == os_type || (os_type == "macos" && value == "osx") {
                            rule_matches = true;
                        }
                    },
                    "arch" => {
                        let os_arch = env::consts::ARCH;
                        if value == os_arch || (value == "x86" && os_arch == "x86_64") {
                            rule_matches = true;
                        }
                    },
                    "version" => {/*TODO: Check version of os to make sure it matches*/},
                    _ => unimplemented!("Unknown rule map key: {}", key),
                }
            }
            // Check if we allow or disallow this downloadable
            match rule.action.as_str() {
                "allow" => rule_matches,
                "disallow" => !rule_matches,
                _ => unimplemented!("Unknwon rule action: {}", rule.action),
            }
        },
    }
}

fn rules_match(rules: &[Rule]) -> bool {
    let mut result = false;
    for rule in rules {
        if rule_matches(rule) {
            result = true;
        } else {
            return false;
        }
    }
    result
}

// HACK: This key generation to get the java version is not optimal and could
//       use to be redone. This uses architecture to map to known java manifest versions.
//       If the manifest ever changes this function most likely needs to be updated. 
fn determine_key_for_java_manifest<'a>(java_version_manifest_map: &HashMap<String, JavaManifest>) -> &'a str {
    let os = env::consts::OS;
    let key = if os == "macos" {
        "mac-os"
    } else {
        os
    };

    if java_version_manifest_map.contains_key(key) {
        return key;
    }
    let architecture = env::consts::ARCH;
    match key {
        "linux" => {
            if architecture == "x86" {"linux-i386"} else {key}
        },
        "mac-os" => {
            if architecture == "arm" {"mac-os-arm64"} else {key}
        },
        "windows" => {
            if architecture == "x86" {"windows-x86"} 
            else if architecture == "x86_64" {"windows-x64"} 
            else {unreachable!("Unexpected windows architecture: {}", architecture)}
        },
        _ => {unreachable!("Unknown java version os: {}. Expected `linux`, `mac-os` or `windows`", key)},
    }
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

    let start = Instant::now();

    let version = resource_manager.download_vanilla_version(&selected).await?;

    let libraries: Vec<Library> = version.libraries.into_iter().filter_map(|lib| {
        // If we have any rules... 
        if let Some(rules) = &lib.rules {
            // and the rules dont match
            if !rules_match(&rules) {
                // remove
                None
            } else {
                // Otherwise keep lib in download list
                Some(lib)
            }
        } else {
            // Otherwise keep lib in download list
            Some(lib)
        }
    }).collect();

    let lib_paths = resource_manager.download_libraries(&libraries).await?;
    
    let game_jar_path = resource_manager.download_game_jar(JarType::Client, &version.downloads.client, &version.id).await?;

    let java_path = resource_manager.download_java_version(&version.java_version.component, version.java_version.major_version).await?;

    resource_manager.download_logging_configurations(&version.logging.client.file).await?;
    
    resource_manager.download_assets(&version.asset_index).await?;
    info!("Finished download instance in {}ms", start.elapsed().as_millis());

    // https://stackoverflow.com/questions/62186871/how-to-correctly-use-peek-in-rust
    construct_arguments(&version.arguments, &lib_paths, &game_jar_path);
    Ok(())
}

fn construct_arguments(arguments: &LaunchArguments, library_paths: &[PathBuf], game_jar_path: &Path) -> Vec<String> {
    // Vec could be 'with_capacity' if we calculate capacity first.
    let mut formatted_arguments: Vec<String> = Vec::new();

    for jvm_arg in arguments.jvm.iter() {
        match jvm_arg {
            Argument::Arg(flag) => {
                let sub_arg = substitute_arg(&flag, &library_paths, &game_jar_path);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => flag.into(),
                });
            },
            Argument::ConditionalArg { rules, value } =>  {
                if !rules_match(&rules) {
                    continue;
                }
                for arg in value {
                    let sub_arg = substitute_arg(&arg, &library_paths, &game_jar_path);
                    formatted_arguments.push(match sub_arg {
                        Some(argument) => argument,
                        None => arg.into(),
                    });
                }
            },
        }
    }
    println!("HERE: {:#?}", formatted_arguments);
    formatted_arguments
}

const LAUNCHER_NAME: &str = "Autmc";
const LAUNCHER_VERSION: &str = "1.0.0";

fn substitute_arg(arg: &str, library_paths: &[PathBuf], game_jar_path: &Path) -> Option<String> {
    let substr_start = arg.chars().position(|c| c == '$');
    let substr_end = arg.chars().position(|c| c == '}');
    let classpath_strs: Vec<&str> = library_paths.into_iter().map(|path| path_to_utf8_str(path)).collect();

    if let (Some(start), Some(end)) = (substr_start, substr_end) {
        let substr = &arg[start..=end];
        info!("Substituting {}", &substr);
        match substr {
            // JVM arguments
            "${natives_directory}" => {Some("".into())},
            "${launcher_name}" => Some(arg.replace(substr, LAUNCHER_NAME)),
            "${launcher_version}" => Some(arg.replace(substr, LAUNCHER_VERSION)),
            "${classpath}" => Some(arg.replace(substr, &format!("\"{}\":\"{}\"", classpath_strs.join("\":\""), path_to_utf8_str(game_jar_path)))),
            // Game arguments
            "${auth_player_name}" => {Some("".into())},
            "${version_name}" => {Some("".into())},
            "${game_directory}" => {Some("".into())},
            "${assets_root}" => {Some("".into())},
            "${assets_index_name}" => {Some("".into())},
            "${auth_uuid}" => {Some("".into())},
            "${auth_access_token}" => {Some("".into())},
            "${clientid}" => {Some("".into())},
            "${auth_xuid}" => {Some("".into())},
            "${user_type}" => {Some("".into())},
            "${version_type}" => {Some("".into())},
            "${resolution_width}" => {Some("".into())},
            "${resolution_height}" => {Some("".into())},
            "${path}" => {Some("".into())},
            _ => {
                None
            },
        }
    } else {
        None
    }
}

fn path_to_utf8_str(path: &Path) -> &str {
    match path.to_str() {
        Some(s) => s,
        None => {
            error!("Retrieved invalid utf8 string from path: {}", path.display());
            "__INVALID_UTF8_STRING__"
        },
    }
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
    logging_dir: PathBuf,
    asset_dir: PathBuf,
    java_dir: PathBuf,
    vanilla_manifest: Option<VanillaManifest>,
    // TODO: Forge and Fabric manifests.
}

impl ResourceManager {
    pub fn new(app_dir: &Path) -> Self {
        Self {
            app_dir: app_dir.into(),
            version_dir: app_dir.join("versions"),
            libraries_dir: app_dir.join("libraries"),
            logging_dir: app_dir.join("logging"),
            asset_dir: app_dir.join("assets"),
            java_dir: app_dir.join("java"),
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
            Err(ManifestError::ResourceError("Trying to access vanilla manifest but it is not downloaded yet.".into()))
        }
    }

    async fn download_libraries(&self, libraries: &[Library]) -> ManifestResult<Vec<PathBuf>> {
        info!("Downloading {} libraries...", libraries.len());
        if !self.libraries_dir.exists() {
            fs::create_dir(&self.libraries_dir)?;
        }

        let start = Instant::now();
        let x = download_all_callback(&libraries, &self.libraries_dir, |bytes, lib| {
            // FIXME: Removing file hashing makes the downloads MUCH faster. Only because of a couple slow hashes, upwards of 1s each
            if !validate_hash(&bytes, &lib.hash()) {
                let err = format!("Error downloading {}, invalid hash.", &lib.url());
                error!("{}", err);
                return Err(DownloadError::InvalidFileHashError(err));
            }
            let path = lib.path(&self.libraries_dir);
            let mut file = File::create(&path)?;
            file.write_all(&bytes)?;
            Ok(())
        }).await;
        info!("Successfully downloaded libraries in {}ms", start.elapsed().as_millis());
        let mut file_paths: Vec<PathBuf> = Vec::with_capacity(libraries.len());
        for lib in libraries {
            file_paths.push(lib.path(&self.libraries_dir));
        }
        Ok(file_paths)
    }

    /// Downloads a game jar (client or server) to ${app_dir}/versions/(client|server)/${version_id}.jar
    async fn download_game_jar(&self, jar_type: JarType, download: &DownloadMetadata, version_id: &str) -> ManifestResult<PathBuf> {
        let jar_str = match jar_type {
            JarType::Client => "client",
            JarType::Server => "server",
        };
        // Create all dirs in path to file location.
        let dir_path = &self.version_dir.join(version_id);
        fs::create_dir_all(dir_path)?;

        let path = dir_path.join(format!("{}.jar", &jar_str));
        let valid_hash = &download.sha1;
        // Check if the file exists and the hash matches the download's sha1.
        if !validate_file_hash(&path, valid_hash) {
            info!("Downloading {} {} jar", version_id, jar_str);
            let bytes = download_bytes_from_url(&download.url).await?;
            if !validate_hash(&bytes, valid_hash) {
                let err = format!("Error downloading {} {} jar, invalid hash.", version_id, jar_str); 
                error!("{}", err);
                return Err(ManifestError::InvalidFileDownload(err));
            }
            let mut file = File::create(&path)?;
            file.write_all(&bytes)?;
        }
        Ok(path)
    }

    async fn download_java_version(&self, java_component: &str, _java_version: u32) -> ManifestResult<PathBuf> {
        info!("Downloading java version manifest");
        let java_version_manifest: HashMap<String, JavaManifest> = download_json_object(JAVA_VERSION_MANIFEST).await?;
        let manifest_key = determine_key_for_java_manifest(&java_version_manifest);

        let java_manifest = &java_version_manifest.get(manifest_key).unwrap();
        let runtime_opt = match java_component {
            "java-runtime-alpha" => &java_manifest.java_runtime_alpha,
            "java-runtime-beta" => &java_manifest.java_runtime_beta,
            "java-runtime-gamma" => &java_manifest.java_runtime_gamma,
            "jre-legacy" => &java_manifest.jre_legacy,
            "minecraft-java-exe" => &java_manifest.minecraft_java_exe,
            _ => unreachable!("No such runtime found for java component: {}", &java_component),
        };
        info!("Downloading runtime: {:#?}", runtime_opt);
        match runtime_opt {
            Some(runtime) => {
                // let runtime_manifest = &runtime.manifest;
                Ok(self.download_java_from_runtime_manifest(&runtime).await?)
            },
            None =>  {
                let s = format!("Java runtime is empty for component {}", &java_component);
                error!("{}", s);
                //TODO: New error type?
                return Err(ManifestError::VersionRetrievalError(s));
            },
        }
    }

    // TODO: Fix file path to include the `name` of the java version being downloaded. Make sure things are marked executable if specified in "executable"
    // FIXME: Use an indexmap instead of a hashmap. Complete this process in a single pass since the index map is ordered correctly.
    //        The correct order is important since it will create dirs before creating files in those dirs. 
    async fn download_java_from_runtime_manifest(&self, manifest: &JavaRuntime) -> ManifestResult<PathBuf> {
        info!("Downloading java runtime manifset");
        let version_manifest: JavaRuntimeManifest = download_json_object(&manifest.manifest.url).await?;
        let base_path = &self.java_dir.join(&manifest.version.name);

        let mut files: Vec<JavaRuntimeFile> = Vec::new();
        // Links is a Vec<(Path, Target)> 
        let mut links: Vec<(String, String)> = Vec::new();
        // Create directories first and save the remaining.
        for jrt in version_manifest.files {
            match jrt {
                JavaRuntimeType::File(jrt_file) => files.push(jrt_file),
                JavaRuntimeType::Directory(dir) => {
                    let path = &base_path.join(dir);
                    fs::create_dir_all(path)?;
                },
                JavaRuntimeType::Link { path, target } => links.push((path, target)),
            }
        }
        
        // Next download files.
        // FIXME: Currently downloading `raw` files, switch to lzma and decompress locally. 
        info!("Downloading all java files.");
        let start = Instant::now();
        let x = download_all_callback(&files, &base_path, |bytes, jrt| {
            if !validate_hash(&bytes, &jrt.hash()) {
                let err = format!("Error downloading {}, invalid hash.", &jrt.url());
                error!("{}", err);
                return Err(DownloadError::InvalidFileHashError(err));
            }
            let path = jrt.path(&base_path);
            let mut file = File::create(&path)?;
            file.write_all(&bytes)?;
            // TODO: Ignoring file permissions currently, theres an "exetutable" field thats unused.
            Ok(())
        }).await;
        info!("Downloaded java in {}ms", start.elapsed().as_millis());

        // Finally create links 
        for link in links {
            let to = &base_path.join(link.0);
            if !to.exists() {
                // Cant fail since the dirs were made before
                let dir_path = to.parent().unwrap().join(link.1);
                let from = dir_path.canonicalize()?;
                debug!("Creating hard link between {} and {}", from.display(), to.display());
                // Create link FROM "target" TO "path" 
                fs::hard_link(from, to)?;
            }
        }
        let java_path = base_path.join("/bin/java");
        info!("Using java path: {}", java_path.display());
        Ok(java_path)
    }

    /// Downloads a logging configureation into ${app_dir}/logging/${logging_configuration.id}
    async fn download_logging_configurations(&self, logging_file: &ClientLoggerFile) -> ManifestResult<()> {
        fs::create_dir_all(&self.logging_dir)?;
        let path = self.logging_dir.join(format!("{}", &logging_file.id));
        let valid_hash = &logging_file.metadata.sha1;

        if !validate_file_hash(&path, valid_hash) {
            info!("Downloading logging configuration {}", logging_file.id);
            let bytes = download_bytes_from_url(&logging_file.metadata.url).await?;
            if !validate_hash(&bytes, valid_hash) {
                let err = format!("Error downloading logging configuration {}, invalid hash.", logging_file.id); 
                error!("{}", err);
                return Err(ManifestError::InvalidFileDownload(err));
            }
            let mut file = File::create(path)?;
            file.write_all(&bytes)?;
        }
        Ok(())
    }

    //TODO: This probably needs to change a little to support "legacy" versions < 1.7
    async fn download_assets(&self, asset_index: &AssetIndex) -> ManifestResult<()> {
        let asset_object: AssetObject = download_json_object(&asset_index.metadata.url).await?;
        info!("Downloading {} assets", &asset_object.objects.len());

        let start = Instant::now();

        let x = download_all_callback(&asset_object.objects, &self.asset_dir, |bytes, asset| {
            if !validate_hash(&bytes, &asset.hash()) {
                let err = format!("Error downloading asset {}, invalid hash.", &asset.name()); 
                error!("{}", err);
                return Err(DownloadError::InvalidFileHashError(err));
            }
            let mut file = File::create(&asset.path(&self.asset_dir))?;
            file.write_all(&bytes)?;
            Ok(())
        }).await;
        info!("Finished downloading assets in {}ms - {:#?}", start.elapsed().as_millis(), &x);
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
        let dir_path = &self.version_dir.join(version_id);
        fs::create_dir_all(dir_path)?;

        let path = &dir_path.join(format!("{}.json", version_id));
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    }

}
