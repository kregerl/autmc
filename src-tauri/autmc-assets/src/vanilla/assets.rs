use super::deserializers;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
/// The launch arguments and metadata for a given vanilla version.
// REVIEW: I believe this response is different for older versions of the game. versions < 1.13
pub struct VanillaVersion {
    #[serde(alias = "minecraftArguments")]
    pub arguments: LaunchArguments,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndex,
    // assets: String,
    // #[serde(rename = "complianceLevel")]
    // compliance_level: Option<u32>,
    pub downloads: GameDownloads,
    pub id: String,
    #[serde(rename = "javaVersion")]
    // FIXME: 1.6.4 and older do not provide a java version.. set to java 8 if not provided.
    pub java_version: Option<JavaVersion>,
    pub libraries: Vec<Library>,
    pub logging: Option<Logging>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    // #[serde(rename = "minimumLauncherVersion")]
    // min_launcher_version: u32,
    // #[serde(rename = "releaseTime")]
    // release_time: String,
    // time: String,
    // #[serde(rename = "type")]
    // version_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum LaunchArguments {
    LaunchArguments112(String),
    LaunchArguments113(LaunchArguments113),
}

#[derive(Debug, Deserialize)]
pub struct LaunchArguments113 {
    pub game: Vec<Argument>,
    // Optional since some older forge versions( < 1.15.2) only have game args
    pub jvm: Option<Vec<Argument>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    Arg(String),
    ConditionalArg {
        rules: Vec<Rule>,
        #[serde(
            rename = "value",
            deserialize_with = "deserializers::string_or_strings_as_vec"
        )]
        values: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    #[serde(flatten)]
    pub rule_type: Option<RuleType>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum RuleType {
    #[serde(rename = "features")]
    Features(HashMap<String, bool>),
    #[serde(rename = "os")]
    OperatingSystem(HashMap<String, String>),
}

#[derive(Debug, Deserialize)]
pub struct AssetIndex {
    pub id: String,
    #[serde(flatten)]
    pub metadata: DownloadMetadata,
    // #[serde(rename = "totalSize")]
    // total_size: u32,
}

#[derive(Debug, Deserialize)]
pub struct GameDownloads {
    pub client: DownloadMetadata,
    pub client_mappings: Option<DownloadMetadata>,
    // Optional for mc version 1.1 and older.
    pub server: Option<DownloadMetadata>,
    pub server_mappings: Option<DownloadMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadMetadata {
    sha1: String,
    size: u32,
    url: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    pub downloads: LibraryDownloads,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
    extract: Option<LibraryExtraction>,
    natives: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Artifact {
    path: String,
    #[serde(flatten)]
    metadata: DownloadMetadata,
}

// TODO: Possible there is an "include" too.
#[derive(Debug, Clone, Deserialize)]
pub struct LibraryExtraction {
    pub exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
// TODO: What about server logging?
pub struct Logging {
    pub client: ClientLogger,
}

#[derive(Debug, Deserialize)]
struct ClientLoggerFile {
    id: String,
    #[serde(flatten)]
    metadata: DownloadMetadata,
}

#[derive(Debug, Deserialize)]
pub struct ClientLogger {
    pub argument: String,
    file: ClientLoggerFile,
    // #[serde(rename = "type")]
    // logger_type: String,
}

#[derive(Debug)]
pub enum JarType {
    Client,
    Server,
}
