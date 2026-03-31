use std::{collections::HashMap, path::{Path, PathBuf}};

use super::{assets::DownloadMetadata, deserializers};
use crate::consts::VANILLA_MANIFEST_URL;
use autmc_downloader::Downloadable;
use indexmap::IndexMap;
use log::{info, warn};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
/// Struct holding everything returned in the vanilla manifest json.
pub struct VanillaManifest {
    // latest: VanillaLatest,
    #[serde(deserialize_with = "deserializers::as_version_map")]
    pub versions: IndexMap<String, VanillaManifestVersion>,
}

#[derive(Debug, Deserialize)]
/// The version metadata returned in the manifest request.
pub struct VanillaManifestVersion {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    // time: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    pub sha1: String,
    // #[serde(rename = "complianceLevel")]
    // compliance_level: u32,
}

pub async fn download_vanilla_manifest() -> reqwest::Result<VanillaManifest> {
    info!("Downloading vanilla manifest");
    let client = reqwest::Client::new();
    let vanilla_response = client.get(VANILLA_MANIFEST_URL).send().await?;
    vanilla_response.json::<VanillaManifest>().await
}


#[derive(Debug, Deserialize)]
pub struct JavaRuntimeVersion {
    pub name: String,
    // released: String,
}

#[derive(Debug, Deserialize)]
pub struct JavaRuntime {
    // availability: JavaRuntimeAvailability,
    pub manifest: DownloadMetadata,
    pub version: JavaRuntimeVersion,
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
        // We know we have one element, get it
        Ok(runtimes.into_iter().next())
    }
}

#[derive(Debug, Deserialize)]
struct JavaRuntimeDownload {
    // lzma: Option<DownloadMetadata>,
    raw: DownloadMetadata,
}

#[derive(Debug, Deserialize)]
pub struct JavaRuntimeFile {
    path: String,
    downloads: JavaRuntimeDownload,
    pub executable: bool,
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

