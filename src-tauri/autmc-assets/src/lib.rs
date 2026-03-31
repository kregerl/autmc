mod consts;
mod fabric;
mod forge;
mod vanilla;

use std::{io::{self, Read}, path::Path, string::FromUtf8Error};
use autmc_downloader::DownloadError;
use serde::Serialize;
use zip::{read::ZipFile, result::ZipError};

pub use fabric::{download_fabric_manifest, FabricLoaderManifest, FabricLoaderVersion};
pub use forge::{download_forge_manifest, ForgeManifest};
use log::error;
pub use vanilla::{*};
pub use fabric::{*};
pub use forge::{*};
// pub use vanilla::{download_vanilla_manifest, VanillaManifest, VanillaManifestVersion, Argument, Rule,RuleType};

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

pub(crate) fn bytes_from_zip_file(file: ZipFile) -> Vec<u8> {
    file.bytes()
        .filter_map(|byte| match byte {
            Ok(b) => Some(b),
            Err(_) => None,
        })
        .collect()
}

pub fn maven_to_vec(
    maven_artifact: &str,
    append_str: Option<&str>,
    force_extension: Option<&str>,
) -> Vec<String> {
    let splits: Vec<&str> = maven_artifact.split(':').collect();
    let file_name_ending = if splits.get(3).is_some() {
        format!("{}-{}", splits[2], splits[3])
    } else {
        splits[2].into()
    };

    let full_file_name = if file_name_ending.contains('@') {
        file_name_ending.replace('@', ".")
    } else {
        format!(
            "{}{}{}",
            file_name_ending,
            if let Some(append) = append_str {
                append
            } else {
                ""
            },
            if let Some(ext) = force_extension {
                ext
            } else {
                ".jar"
            },
        )
    };

    let mut result = Vec::new();
    result.append(&mut splits[0].split('.').collect::<Vec<&str>>());
    result.push(splits[1]);
    result.push(splits[2].split('@').collect::<Vec<&str>>()[0]);
    let final_name = format!("{}-{}", splits[1], full_file_name);
    result.push(&final_name);

    result.iter().map(|s| (*s).to_owned()).collect()
}

/// Converts a path into a utf8 compatible string. If the string is not utf8 compatible then
/// it is set to an obvious error str: '__INVALID_UTF8_STRING__'
pub fn path_to_utf8_str(path: &Path) -> &str {
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

#[cfg(target_family = "unix")]
pub fn get_classpath_separator() -> &'static str {
    ":"
}

#[cfg(target_family = "windows")]
pub fn get_classpath_separator() -> &'static str {
    ";"
}

#[cfg(target_family = "unix")]
pub fn get_directory_separator() -> &'static str {
    "/"
}

#[cfg(target_family = "windows")]
pub fn get_directory_separator() -> &'static str {
    "\\"
}