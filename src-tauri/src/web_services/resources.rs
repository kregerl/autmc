use std::{collections::HashMap, path::{Path, PathBuf}};

use indexmap::IndexMap;
use log::warn;
use serde::{Deserialize, Serialize, Deserializer, de::{Visitor, Error, SeqAccess}};

use super::{downloader::Downloadable, consts::VANILLA_ASSET_BASE_URL};

#[derive(Debug, Deserialize)]
/// The version metadata returned in the manifest request.
pub struct VanillaManifestVersion {
    id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    time: String,
    #[serde(rename = "releaseTime")]
    release_time: String,
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
pub enum RuleType {
    #[serde(rename = "features")]
    Features(HashMap<String, bool>),
    #[serde(rename = "os")]
    OperatingSystem(HashMap<String, String>),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Rule {
    pub action: String,
    #[serde(flatten)]
    pub rule_type: RuleType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Argument {
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
pub struct LaunchArguments {
    pub game: Vec<Argument>,
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadMetadata {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Asset {
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
pub struct AssetObject {
    #[serde(deserialize_with = "to_asset_vec")]
    pub objects: Vec<Asset>,
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
pub struct AssetIndex {
    id: String,
    #[serde(flatten)]
    pub metadata: DownloadMetadata,
    #[serde(rename = "totalSize")]
    total_size: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameDownloads {
    pub client: DownloadMetadata,
    pub client_mappings: DownloadMetadata,
    pub server: DownloadMetadata,
    pub server_mappings: DownloadMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
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
    pub rules: Option<Vec<Rule>>,
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
pub struct ClientLoggerFile {
    pub id: String,
    #[serde(flatten)]
    pub metadata: DownloadMetadata,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientLogger {
    argument: String,
    pub file: ClientLoggerFile,
    #[serde(rename = "type")]
    logger_type: String,
}

#[derive(Debug, Deserialize, Serialize)]
// TODO: What about server logging?
pub struct Logging {
    pub client: ClientLogger,
}

#[derive(Debug, Deserialize, Serialize)]
/// The launch arguments and metadata for a given vanilla version.
// REVIEW: I believe this response is different for older versions of the game. versions < 1.13
pub struct VanillaVersion {
    pub arguments: LaunchArguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    assets: String,
    #[serde(rename = "complianceLevel")]
    compliance_level: u32,
    pub downloads: GameDownloads,
    pub id: String,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersion,
    pub libraries: Vec<Library>,
    pub logging: Logging,
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
pub struct JavaRuntimeVesion {
    pub name: String,
    released: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaRuntime {
    availability: JavaRuntimeAvailability,
    pub manifest: DownloadMetadata,
    pub version: JavaRuntimeVesion,
}

#[derive(Debug, Deserialize)]
pub struct JavaManifest {
    #[serde(
        rename = "java-runtime-alpha",
        deserialize_with = "deserialize_java_runtime"
    )]
    pub java_runtime_alpha: Option<JavaRuntime>,
    #[serde(
        rename = "java-runtime-beta",
        deserialize_with = "deserialize_java_runtime"
    )]
    pub java_runtime_beta: Option<JavaRuntime>,
    #[serde(
        rename = "java-runtime-gamma",
        deserialize_with = "deserialize_java_runtime"
    )]
    pub java_runtime_gamma: Option<JavaRuntime>,
    #[serde(rename = "jre-legacy", deserialize_with = "deserialize_java_runtime")]
    pub jre_legacy: Option<JavaRuntime>,
    #[serde(
        rename = "minecraft-java-exe",
        deserialize_with = "deserialize_java_runtime"
    )]
    pub minecraft_java_exe: Option<JavaRuntime>,
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
        // We know we have atleast one element
        Ok(runtimes.into_iter().nth(0))
    }
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeDownload {
    lzma: Option<DownloadMetadata>,
    raw: DownloadMetadata,
}

#[derive(Debug, Deserialize)]
pub struct JavaRuntimeFile {
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
pub enum JavaRuntimeType {
    #[serde(rename = "file")]
    File(JavaRuntimeFile),
    #[serde(rename = "directory")]
    Directory(String),
    #[serde(rename = "link")]
    Link { path: String, target: String },
}

#[derive(Debug, Deserialize)]
pub struct JavaRuntimeManifest {
    #[serde(deserialize_with = "to_java_runtime_vec")]
    pub files: Vec<JavaRuntimeType>,
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