use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use indexmap::IndexMap;
use log::{debug, error, info, warn};
use serde::{
    de::{Error, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
    consts::{JAVA_VERSION_MANIFEST, LAUNCHER_NAME, LAUNCHER_VERSION, VANILLA_ASSET_BASE_URL},
    state::{
        account_manager::{Account, AccountState},
        resource_manager::{ManifestError, ManifestResult, ResourceState},
    },
    web_services::downloader::{
        download_all_callback, download_bytes_from_url, download_json_object, validate_hash,
        DownloadError,
    },
};

use super::downloader::{validate_file_hash, Downloadable};

#[derive(Debug, Deserialize)]
/// The version metadata returned in the manifest request.
pub struct VanillaManifestVersion {
    id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32,
}

#[derive(Debug, Deserialize)]
/// Struct holding everything returned in the vanilla manifest json.
pub struct VanillaManifest {
    // latest: VanillaLatest,
    #[serde(deserialize_with = "as_version_map")]
    pub versions: IndexMap<String, VanillaManifestVersion>,
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

#[derive(Debug, Deserialize, Serialize)]
enum RuleType {
    #[serde(rename = "features")]
    Features(HashMap<String, bool>),
    #[serde(rename = "os")]
    OperatingSystem(HashMap<String, String>),
}

#[derive(Debug, Deserialize, Serialize)]
struct Rule {
    pub action: String,
    #[serde(flatten)]
    pub rule_type: RuleType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Argument {
    Arg(String),
    ConditionalArg {
        rules: Vec<Rule>,
        #[serde(rename = "value", deserialize_with = "string_or_strings_as_vec")]
        values: Vec<String>,
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

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(vec![v.into()])
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
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
    jvm: Vec<Argument>,
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
    size: u32,
}

impl Downloadable for Asset {
    fn name(&self) -> &str {
        &self.path
    }

    fn url(&self) -> String {
        let first_two_chars = &self.hash.split_at(2);
        let url = format!(
            "{}/{}/{}",
            VANILLA_ASSET_BASE_URL, &first_two_chars.0, &self.hash
        );
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
    objects: Vec<Asset>,
}

fn to_asset_vec<'de, D>(deserializer: D) -> Result<Vec<Asset>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Debug, Deserialize)]
    struct TmpAsset {
        hash: String,
        size: u32,
    }

    let asset_map: HashMap<String, TmpAsset> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(asset_map.len());
    for (path, tmp_asset) in asset_map {
        result.push(Asset {
            path,
            hash: tmp_asset.hash,
            size: tmp_asset.size,
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
struct Library {
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
pub struct VanillaVersion {
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
pub enum JarType {
    Client,
    Server,
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeAvailability {
    group: u32,
    progress: u32,
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
    #[serde(
        rename = "java-runtime-alpha",
        deserialize_with = "deserialize_java_runtime"
    )]
    java_runtime_alpha: Option<JavaRuntime>,
    #[serde(
        rename = "java-runtime-beta",
        deserialize_with = "deserialize_java_runtime"
    )]
    java_runtime_beta: Option<JavaRuntime>,
    #[serde(
        rename = "java-runtime-gamma",
        deserialize_with = "deserialize_java_runtime"
    )]
    java_runtime_gamma: Option<JavaRuntime>,
    #[serde(rename = "jre-legacy", deserialize_with = "deserialize_java_runtime")]
    jre_legacy: Option<JavaRuntime>,
    #[serde(
        rename = "minecraft-java-exe",
        deserialize_with = "deserialize_java_runtime"
    )]
    minecraft_java_exe: Option<JavaRuntime>,
}

fn deserialize_java_runtime<'de, D>(deserializer: D) -> Result<Option<JavaRuntime>, D::Error>
where
    D: Deserializer<'de>,
{
    let runtimes: Vec<JavaRuntime> = Deserialize::deserialize(deserializer)?;
    if runtimes.len() > 1 {
        warn!(
            "Got more java runtimes than expected. Expected 1 but got {}",
            runtimes.len()
        );
        Ok(None)
    } else {
        // We know we have one element, get it
        Ok(runtimes.into_iter().nth(0))
    }
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
    Link { path: String, target: String },
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
            executable: bool,
        },
        #[serde(rename = "directory")]
        Directory,
        #[serde(rename = "link")]
        Link { target: String },
    }
    let jrt_map: HashMap<String, TmpJavaRuntimeType> = Deserialize::deserialize(deserializer)?;
    println!("HERE: {:#?}", jrt_map);
    let mut result = Vec::with_capacity(jrt_map.len());
    for (path, tmp_jrt) in jrt_map {
        result.push(match tmp_jrt {
            TmpJavaRuntimeType::File {
                downloads,
                executable,
            } => JavaRuntimeType::File(JavaRuntimeFile {
                path,
                downloads,
                executable,
            }),
            TmpJavaRuntimeType::Directory => JavaRuntimeType::Directory(path),
            TmpJavaRuntimeType::Link { target } => JavaRuntimeType::Link { path, target },
        });
    }
    Ok(result)
}

/// Checks if a single rule matches every case.
/// Returns true when an allow rule matches or a disallow rule does not match.
fn rule_matches(rule: &Rule) -> bool {
    match &rule.rule_type {
        RuleType::Features(_feature_rules) => {
            error!("Implement feature rules for arguments");
            // FIXME: Currently just skipping these
            false
        }
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
                    }
                    "arch" => {
                        let os_arch = env::consts::ARCH;
                        if value == os_arch || (value == "x86" && os_arch == "x86_64") {
                            rule_matches = true;
                        }
                    }
                    "version" => { /*TODO: Check version of os to make sure it matches*/ }
                    _ => unimplemented!("Unknown rule map key: {}", key),
                }
            }
            // Check if we allow or disallow this downloadable
            match rule.action.as_str() {
                "allow" => rule_matches,
                "disallow" => !rule_matches,
                _ => unimplemented!("Unknwon rule action: {}", rule.action),
            }
        }
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
fn determine_key_for_java_manifest<'a>(
    java_version_manifest_map: &HashMap<String, JavaManifest>,
) -> &'a str {
    let os = env::consts::OS;
    let key = if os == "macos" { "mac-os" } else { os };

    if java_version_manifest_map.contains_key(key) {
        return key;
    }
    let architecture = env::consts::ARCH;
    match key {
        "linux" => {
            if architecture == "x86" {
                "linux-i386"
            } else {
                key
            }
        }
        "mac-os" => {
            if architecture == "arm" {
                "mac-os-arm64"
            } else {
                key
            }
        }
        "windows" => {
            if architecture == "x86" {
                "windows-x86"
            } else if architecture == "x86_64" {
                "windows-x64"
            } else {
                unreachable!("Unexpected windows architecture: {}", architecture)
            }
        }
        _ => {
            unreachable!(
                "Unknown java version os: {}. Expected `linux`, `mac-os` or `windows`",
                key
            )
        }
    }
}

fn construct_arguments(
    java_path: &Path,
    main_class: String,
    logging: (String, PathBuf), // (Argument to substitute, Path to substitute with) 
    arguments: &LaunchArguments,
    active_account: &Account,
    library_paths: &[PathBuf],
    game_jar_path: &Path,
    instance_path: &Path,
    mc_version: &str,
    asset_dir: &Path,
    asset_name: &str
) -> Vec<String> {
    // Vec could be 'with_capacity' if we calculate capacity first.
    let mut formatted_arguments: Vec<String> = Vec::new();

    info!("Args: {:#?}", arguments);

    info!("LOGGING PATH: {:?}", &logging.1);

    formatted_arguments.push(format!("\"{}\"", java_path.to_str().unwrap()));
    for jvm_arg in arguments.jvm.iter() {
        match jvm_arg {
            // For normal arguments, check if it has something that should be replaced and replace it 
            Argument::Arg(value) => {
                let sub_arg =
                    substitute_jvm_arg(&value, &library_paths, &instance_path, &game_jar_path);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            // For conditional args, check their rules before adding to formatted_arguments vec
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(&rules) {
                    continue;
                }
                for value in values {
                    let sub_arg =
                        substitute_jvm_arg(&value, &library_paths, &instance_path, &game_jar_path);
                    formatted_arguments.push(match sub_arg {
                        Some(argument) => argument,
                        None => value.into(),
                    });
                }
            }
        }
    }
    if let Some(substr) = get_arg_substring(&logging.0) {
        formatted_arguments.push(logging.0.replace(substr, path_to_utf8_str(&logging.1)));
    }
    formatted_arguments.push(main_class);

    for game_arg in arguments.game.iter() {
        match game_arg {
            Argument::Arg(value) => {
                let sub_arg = substitute_game_arg(&value, &instance_path, &active_account, &mc_version, &asset_dir, asset_name);
                formatted_arguments.push(match sub_arg {
                    Some(argument) => argument,
                    None => value.into(),
                });
            }
            Argument::ConditionalArg { rules, values } => {
                if !rules_match(&rules) {
                    continue;
                }
                for value in values {
                    let sub_arg = substitute_game_arg(&value, &instance_path, &active_account, &mc_version, &asset_dir, asset_name);
                    formatted_arguments.push(match sub_arg {
                        Some(argument) => argument,
                        None => value.into(),
                    });
                }
            }
        }
    }
    println!("HERE: {:#?}", formatted_arguments);
    formatted_arguments
}

// Returns the substring inside the argument if it exists, otherwise None
fn get_arg_substring(arg: &str) -> Option<&str> {
    let substr_start = arg.chars().position(|c| c == '$');
    let substr_end = arg.chars().position(|c| c == '}');

    if let (Some(start), Some(end)) = (substr_start, substr_end) {
        Some(&arg[start..=end])
    } else {
        None
    }
}

// Returns a string with the substituted value in the jvm argument or None if it doesn't apply. 
fn substitute_jvm_arg(
    arg: &str,
    library_paths: &[PathBuf],
    instance_path: &Path,
    game_jar_path: &Path,
) -> Option<String> {
    let substring = get_arg_substring(arg);
    let classpath_strs: Vec<&str> = library_paths
        .into_iter()
        .map(|path| path_to_utf8_str(path))
        .collect();

    if let Some(substr) = substring {
        info!("Substituting {}", &substr);
        match substr {
            "${natives_directory}" => Some(arg.replace(
                substr,
                &format!("\"{}\"", path_to_utf8_str(&instance_path.join("natives"))),
            )),
            "${launcher_name}" => Some(arg.replace(substr, LAUNCHER_NAME)),
            "${launcher_version}" => Some(arg.replace(substr, LAUNCHER_VERSION)),
            "${classpath}" => Some(arg.replace(
                substr,
                &format!(
                    "\"{}\":\"{}\"",
                    classpath_strs.join("\":\""),
                    path_to_utf8_str(game_jar_path)
                ),
            )),
            _ => None,
        }
    } else {
        None
    }
}

fn substitute_game_arg(
    arg: &str,
    instance_path: &Path,
    active_account: &Account,
    mc_version: &str,
    asset_dir: &Path,
    asset_name: &str
) -> Option<String> {
    let substring = get_arg_substring(arg);

    if let Some(substr) = substring {
        info!("Substituting {}", &substr);
        match substr {
            "${auth_player_name}" => Some(arg.replace(substr, &active_account.name)),
            "${version_name}" => Some(arg.replace(substr, &mc_version)), 
            "${game_directory}" => Some(arg.replace(substr, &format!("\"{}\"", path_to_utf8_str(&instance_path)))),
            "${assets_root}" => Some(arg.replace(substr, &format!("\"{}\"", path_to_utf8_str(&asset_dir)))),
            "${assets_index_name}" => Some(arg.replace(substr, &asset_name)),
            "${auth_uuid}" => Some(arg.replace(substr, &active_account.uuid)),
            "${auth_access_token}" => {
                Some(arg.replace(substr, &active_account.minecraft_access_token))
            }
            "${clientid}" => None,  // FIXME: Unknown
            "${auth_xuid}" => None, // FIXME: Unknown
            "${user_type}" => Some(arg.replace(substr, "mojang")), // TODO: Unknown but hardcoded to "mojang" as thats what the gdlauncher example shows
            "${version_type}" => Some(arg.replace(substr, "release")), // TODO: Get from the selected versions type {release or snapshot}
            "${resolution_width}" => None,                  // TODO: Unknown
            "${resolution_height}" => None,                 // TODO: Unknown
            _ => None,
        }
    } else {
        None
    }
}

fn path_to_utf8_str(path: &Path) -> &str {
    match path.to_str() {
        Some(s) => s,
        None => {
            error!(
                "Retrieved invalid utf8 string from path: {}",
                path.display()
            );
            "__INVALID_UTF8_STRING__"
        }
    }
}

async fn download_libraries(
    libraries_dir: &Path,
    libraries: &[Library],
) -> ManifestResult<Vec<PathBuf>> {
    info!("Downloading {} libraries...", libraries.len());
    if !libraries_dir.exists() {
        fs::create_dir(&libraries_dir)?;
    }

    let start = Instant::now();
    let x = download_all_callback(&libraries, &libraries_dir, |bytes, lib| {
        // FIXME: Removing file hashing makes the downloads MUCH faster. Only because of a couple slow hashes, upwards of 1s each
        if !validate_hash(&bytes, &lib.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &lib.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        let path = lib.path(&libraries_dir);
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
        Ok(())
    })
    .await;
    info!(
        "Successfully downloaded libraries in {}ms",
        start.elapsed().as_millis()
    );
    let mut file_paths: Vec<PathBuf> = Vec::with_capacity(libraries.len());
    for lib in libraries {
        file_paths.push(lib.path(&libraries_dir));
    }
    Ok(file_paths)
}

async fn download_game_jar(
    versions_dir: &Path,
    jar_type: JarType,
    download: &DownloadMetadata,
    version_id: &str,
) -> ManifestResult<PathBuf> {
    let jar_str = match jar_type {
        JarType::Client => "client",
        JarType::Server => "server",
    };
    // Create all dirs in path to file location.
    let dir_path = &versions_dir.join(version_id);
    fs::create_dir_all(dir_path)?;

    let path = dir_path.join(format!("{}.jar", &jar_str));
    let valid_hash = &download.sha1;
    // Check if the file exists and the hash matches the download's sha1.
    if !validate_file_hash(&path, valid_hash) {
        info!("Downloading {} {} jar", version_id, jar_str);
        let bytes = download_bytes_from_url(&download.url).await?;
        if !validate_hash(&bytes, valid_hash) {
            let err = format!(
                "Error downloading {} {} jar, invalid hash.",
                version_id, jar_str
            );
            error!("{}", err);
            return Err(ManifestError::InvalidFileDownload(err));
        }
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
    }
    Ok(path)
}

// FIXME: Use an indexmap instead of a hashmap. Complete this process in a single pass since the index map is ordered correctly.
//        The correct order is important since it will create dirs before creating files in those dirs.
async fn download_java_from_runtime_manifest(
    java_dir: &Path,
    manifest: &JavaRuntime,
) -> ManifestResult<PathBuf> {
    info!("Downloading java runtime manifset");
    let version_manifest: JavaRuntimeManifest =
        download_json_object(&manifest.manifest.url).await?;
    let base_path = &java_dir.join(&manifest.version.name);

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
            }
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
    })
    .await;
    info!("Downloaded java in {}ms", start.elapsed().as_millis());

    // Finally create links
    for link in links {
        let to = &base_path.join(link.0);
        if !to.exists() {
            // Cant fail since the dirs were made before
            let dir_path = to.parent().unwrap().join(link.1);
            let from = dir_path.canonicalize()?;
            debug!(
                "Creating hard link between {} and {}",
                from.display(),
                to.display()
            );
            // Create link FROM "target" TO "path"
            fs::hard_link(from, to)?;
        }
    }

    let java_path = base_path.join("bin/java");
    info!("Using java path: {:?}", java_path);
    Ok(java_path)
}

async fn download_java_version(
    java_dir: &Path,
    java_component: &str,
    _java_version: u32,
) -> ManifestResult<PathBuf> {
    info!("Downloading java version manifest");
    let java_version_manifest: HashMap<String, JavaManifest> =
        download_json_object(JAVA_VERSION_MANIFEST).await?;
    let manifest_key = determine_key_for_java_manifest(&java_version_manifest);

    let java_manifest = &java_version_manifest.get(manifest_key).unwrap();
    let runtime_opt = match java_component {
        "java-runtime-alpha" => &java_manifest.java_runtime_alpha,
        "java-runtime-beta" => &java_manifest.java_runtime_beta,
        "java-runtime-gamma" => &java_manifest.java_runtime_gamma,
        "jre-legacy" => &java_manifest.jre_legacy,
        "minecraft-java-exe" => &java_manifest.minecraft_java_exe,
        _ => unreachable!(
            "No such runtime found for java component: {}",
            &java_component
        ),
    };
    info!("Downloading runtime: {:#?}", runtime_opt);
    match runtime_opt {
        Some(runtime) => {
            // let runtime_manifest = &runtime.manifest;
            Ok(download_java_from_runtime_manifest(&java_dir, &runtime).await?)
        }
        None => {
            let s = format!("Java runtime is empty for component {}", &java_component);
            error!("{}", s);
            //TODO: New error type?
            return Err(ManifestError::VersionRetrievalError(s));
        }
    }
}

/// Downloads a logging configureation into ${app_dir}/logging/${logging_configuration.id}
async fn download_logging_configurations(
    logging_dir: &Path,
    logging: &Logging,
) -> ManifestResult<(String, PathBuf)> {
    fs::create_dir_all(&logging_dir)?;
    let path = logging_dir.join(format!("{}", &logging.client.file.id));
    let valid_hash = &logging.client.file.metadata.sha1;

    if !validate_file_hash(&path, valid_hash) {
        info!(
            "Downloading logging configuration {}",
            logging.client.file.id
        );
        let bytes = download_bytes_from_url(&logging.client.file.metadata.url).await?;
        if !validate_hash(&bytes, valid_hash) {
            let err = format!(
                "Error downloading logging configuration {}, invalid hash.",
                logging.client.file.id
            );
            error!("{}", err);
            return Err(ManifestError::InvalidFileDownload(err));
        }
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
    }
    Ok((logging.client.argument.clone(), path))
}

//TODO: This probably needs to change a little to support "legacy" versions < 1.7
async fn download_assets(asset_dir: &Path, asset_index: &AssetIndex) -> ManifestResult<String> {
    let asset_object: AssetObject = download_json_object(&asset_index.metadata.url).await?;
    let asset_index_dir = asset_dir.join("indexes");
    let index_bytes = download_bytes_from_url(&asset_index.metadata.url).await?;
    fs::create_dir_all(&asset_index_dir)?;

    info!("Asset Index ID: {:?}", &asset_index);

    let asset_index_name = format!("{}.json", &asset_index.id);
    let index_path = &asset_index_dir.join(&asset_index_name);
    let mut index_file = File::create(index_path)?;
    index_file.write_all(&index_bytes)?;
    info!("Downloading {} assets", &asset_object.objects.len());

    let start = Instant::now();

    let x = download_all_callback(&asset_object.objects, &asset_dir, |bytes, asset| {
        if !validate_hash(&bytes, &asset.hash()) {
            let err = format!("Error downloading asset {}, invalid hash.", &asset.name());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        let mut file = File::create(&asset.path(&asset_dir))?;
        file.write_all(&bytes)?;
        Ok(())
    })
    .await;
    info!(
        "Finished downloading assets in {}ms - {:#?}",
        start.elapsed().as_millis(),
        &x
    );
    Ok(asset_index.id.clone())
}

pub async fn create_instance(
    selected: String,
    instance_name: String,
    app_handle: &AppHandle<Wry>,
) -> ManifestResult<()> {
    let resource_state: State<ResourceState> = app_handle
        .try_state()
        .expect("`ResourceState` should already be managed.");
    let resource_manager = resource_state.0.lock().await;
    let start = Instant::now();

    let version = resource_manager.download_vanilla_version(&selected).await?;

    let libraries: Vec<Library> = version
        .libraries
        .into_iter()
        .filter_map(|lib| {
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
        })
        .collect();

    let lib_paths = download_libraries(&resource_manager.libraries_dir(), &libraries).await?;

    let game_jar_path = download_game_jar(
        &resource_manager.version_dir(),
        JarType::Client,
        &version.downloads.client,
        &version.id,
    )
    .await?;

    let java_path = download_java_version(
        &resource_manager.java_dir(),
        &version.java_version.component,
        version.java_version.major_version,
    )
    .await?;

    info!("create_instance JVM PATH: {:?}", &java_path);

    let logging =
        download_logging_configurations(&resource_manager.logging_dir(), &version.logging).await?;

    let asset_index = download_assets(&resource_manager.assets_dir(), &version.asset_index).await?;
    info!(
        "Finished download instance in {}ms",
        start.elapsed().as_millis()
    );

    let instance_dir = resource_manager.instances_dir().join(instance_name);
    fs::create_dir_all(&instance_dir)?;

    let account_state: State<AccountState> = app_handle
        .try_state()
        .expect("`AccountState` should already be managed.");
    let account_manager = account_state.0.lock().await;
    // FIXME: Dont unwrap here.
    let active_account = &account_manager.get_active_account().unwrap();

    let x = construct_arguments(
        &java_path,
        version.main_class,
        logging,
        &version.arguments,
        &active_account,
        &lib_paths,
        &game_jar_path,
        &instance_dir,
        &selected,
        &resource_manager.assets_dir(),
        &asset_index
    );
    info!("{}", x.join(" "));
    Ok(())
}
