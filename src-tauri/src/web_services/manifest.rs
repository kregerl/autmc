use std::{path::Path, io::Read};

use log::error;
use zip::read::ZipFile;

pub mod vanilla;
pub mod forge;
pub mod fabric;

pub fn maven_to_vec(maven_artifact: &str, append_str: Option<&str>, force_extension: Option<&str>) -> Vec<String> {
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

pub(crate) fn bytes_from_zip_file(file: ZipFile) -> Vec<u8> {
    file.bytes()
        .filter_map(|byte| match byte {
            Ok(b) => Some(b),
            Err(_) => None,
        })
        .collect()
}

#[cfg(target_family = "unix")]
pub fn get_classpath_separator() -> String {
    ":".into()
}

#[cfg(target_family = "windows")]
pub fn get_classpath_separator() -> String {
    ";".into()
}

#[cfg(target_family = "unix")]
pub fn get_directory_separator() -> String {
    "/".into()
}

#[cfg(target_family = "windows")]
pub fn get_directory_separator() -> String {
    "\\".into()
}