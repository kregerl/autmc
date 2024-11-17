use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
    time::Instant,
};

use crate::state::ManagerFromAppHandle;
use crate::{
    state::instance_manager::{InstanceManager, InstanceState},
    web_services::{
        downloader::{buffered_download_stream, validate_hash_sha1, DownloadError, Downloadable},
        manifest::bytes_from_zip_file,
        resources::{create_instance, InstanceSettings, ModloaderType},
    },
};
use log::{debug, error, info};
use serde::Deserialize;
use tauri::{AppHandle, Manager, State, Wry};
use zip::ZipArchive;

#[derive(Debug, Deserialize)]
struct ModrinthManifest {
    #[serde(rename = "formatVersion")]
    format_version: u32,
    game: String,
    #[serde(rename = "versionId")]
    version_id: String,
    name: String,
    summary: Option<String>,
    files: Vec<ModrinthFile>,
    dependencies: ModrinthDependencies,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    path: String,
    hashes: ModrinthHashes,
    env: Option<ModrinthEnv>,
    // Vec of all possible downloads, if one fails fallback to others
    downloads: Vec<String>,
    #[serde(rename = "fileSize")]
    file_size: u32,
}

impl Downloadable for ModrinthFile {
    fn name(&self) -> &str {
        &self.path
    }

    fn url(&self) -> String {
        // TODO: Fallback to alternate downloads when/if first one fails.
        // Assumes there is always 1 download.
        self.downloads.first().unwrap().into()
    }

    fn hash(&self) -> &str {
        &self.hashes.sha1
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.path)
    }
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha1: String,
    sha512: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthEnv {
    client: String,
    server: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthDependencies {
    minecraft: String,
    #[serde(flatten)]
    modloader_dependency: ModrinthModloaderDependency,
}

#[derive(Debug, Deserialize)]
enum ModrinthModloaderDependency {
    #[serde(rename = "fabric-loader")]
    Fabric(String),
    #[serde(rename = "forge")]
    Forge(String),
}

pub async fn import_modrinth_zip(
    archive: &mut ZipArchive<&File>,
    app_handle: &AppHandle<Wry>,
) -> io::Result<()> {
    info!("Importing modrinth zip...");
    let manifest_bytes = bytes_from_zip_file(archive.by_name("modrinth.index.json").unwrap());
    let manifest: ModrinthManifest = serde_json::from_slice(&manifest_bytes)?;
    debug!("Manifset: {:#?}", manifest);

    let (modloader_version, modloader_type) = match manifest.dependencies.modloader_dependency {
        ModrinthModloaderDependency::Fabric(version) => (version, ModloaderType::Fabric),
        ModrinthModloaderDependency::Forge(version) => (
            format!("{}-{}", manifest.dependencies.minecraft, version),
            ModloaderType::Forge,
        ),
    };

    let settings = InstanceSettings::new(
        manifest.name.clone(),
        manifest.dependencies.minecraft,
        modloader_type,
        modloader_version,
        None,
    );

    create_instance(settings, app_handle, Some("Modrinth"))
        .await
        .unwrap();

    let instance_manager = InstanceManager::from_app_handle(&app_handle).await;

    let instances_dir = instance_manager.instances_dir();
    let instance_dir = instances_dir.join(&manifest.name);

    download_mods_from_modrinth(manifest.files, &instance_dir).await?;

    extract_overrides(&instance_dir, archive)?;
    info!("Succcessfully imported modrinth modpack {}", manifest.name);
    Ok(())
}

async fn download_mods_from_modrinth(
    files: Vec<ModrinthFile>,
    instance_dir: &Path,
) -> io::Result<()> {
    fs::create_dir_all(&instance_dir)?;

    let x = buffered_download_stream(&files, &instance_dir, |bytes, file| {
        if !validate_hash_sha1(bytes, file.hash()) {
            let err = format!("Error downloading {}, invalid hash.", file.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHash(err));
        }
        debug!("Downloading mod: {}", file.name());
        let path = file.path(&instance_dir);
        let mut file = File::create(path)?;
        file.write_all(bytes)?;
        Ok(())
    })
    .await;

    Ok(())
}

fn extract_overrides(instance_dir: &Path, archive: &mut ZipArchive<&File>) -> io::Result<()> {
    info!("Extracting overrides into {:#?}", instance_dir);
    const OVERRIDES: &str = "overrides";
    for i in 0..archive.len() {
        let zip_file = archive.by_index(i)?;
        let name = zip_file.enclosed_name().unwrap().to_path_buf();
        if name.starts_with(OVERRIDES) && zip_file.is_file() {
            let timer = Instant::now();

            let base_path = name.strip_prefix(OVERRIDES).unwrap();
            let path = instance_dir.join(base_path);
            let bytes = bytes_from_zip_file(zip_file);

            let parent = path.parent();
            if let Some(parent_dir) = parent {
                if !parent_dir.exists() {
                    fs::create_dir_all(parent_dir)?;
                }
            }
            let mut file = File::create(&path)?;
            file.write_all(&bytes)?;
            // TODO: speed up background.png extraction speed
            debug!(
                "Extracting {:#?} took {}ms for {} bytes",
                path,
                timer.elapsed().as_millis(),
                bytes.len()
            );
        }
    }

    Ok(())
}
