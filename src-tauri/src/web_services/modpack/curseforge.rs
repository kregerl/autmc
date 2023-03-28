use core::arch;
use std::{
    fs::{File, self},
    io::{self, Write},
    path::{Path, PathBuf}, time::Instant,
};

use log::{debug, error};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use serde_json::json;
#[cfg(test)]
use tauri::async_runtime::block_on;
use tauri::{AppHandle, Manager, State, Wry};
use tempdir::TempDir;
use zip::{read::ZipFile, ZipArchive};

use crate::{
    consts::{CURSEFORGE_API_URL, CURSEFORGE_FORGECDN_URL},
    state::instance_manager::InstanceState,
    web_services::{
        downloader::{
            boxed_buffered_download_stream, validate_hash_sha1, DownloadError, DownloadResult,
            Downloadable, download_json_object, download_json_object_with_headers,
        },
        manifest::bytes_from_zip_file,
    },
};

// -----------------------------
// START: Curseforge Zip Files
// -----------------------------
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseforgeManifest {
    minecraft: CurseforgeGameInformation,
    manifest_type: String,
    manifest_version: u32,
    name: String,
    version: String,
    author: String,
    pub files: Vec<CurseforgeFile>,
    overrides: String,
}

impl CurseforgeManifest {
    pub fn get_vanilla_version(&self) -> &str {
        &self.minecraft.version
    }

    pub fn get_modloaders(&self) -> &[Modloader] {
        &self.minecraft.modloaders
    }

    pub fn get_modpack_name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeGameInformation {
    version: String,
    #[serde(rename = "modLoaders")]
    modloaders: Vec<Modloader>,
}

#[derive(Debug, Deserialize)]
pub struct Modloader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CurseforgeFile {
    #[serde(rename = "projectID")]
    project_id: u32,
    #[serde(rename = "fileID")]
    file_id: u32,
    required: bool,
}

pub fn extract_curseforge_zip(archive: &mut ZipArchive<&File>) -> io::Result<CurseforgeManifest> {
    let manifest_bytes = bytes_from_zip_file(archive.by_name("manifest.json")?);

    Ok(serde_json::from_slice(&manifest_bytes)?)
}

pub fn extract_overrides(instance_path: &Path, archive: &mut ZipArchive<&File>) -> io::Result<()> {
    debug!("Extracting overrides into {:#?}", instance_path);
    for i in 0..archive.len() {
        let zip_file = archive.by_index(i)?;
        let name = zip_file.enclosed_name().unwrap().to_path_buf();
        if name.starts_with("overrides") && zip_file.is_file() {
            let timer = Instant::now();

            let base_path = name.strip_prefix("overrides").unwrap();
            let path = instance_path.join(base_path);
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
            debug!("Extracting {:#?} took {}ms for {} bytes", path, timer.elapsed().as_millis(), bytes.len());
        }

    }
    Ok(())
}

// -----------------------------
// END: Curseforge Zip Files
// -----------------------------

// -----------------------------
// START: Common Curseforge Structs
// -----------------------------

#[derive(Debug, Deserialize)]
struct CurseforgeHash {
    value: String,
    // Valid hash algos: 1 = Sha1, 2 = Md5
    algo: u8,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSortableGameVersion {
    game_version_name: String,
    game_version_padded: String,
    game_version: Option<String>,
    game_version_release_date: String,
    game_version_type_id: Option<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeDependency {
    mod_id: u32,
    relation_type: u32,
}

// -----------------------------
// END: Common Curseforge Structs
// -----------------------------

// -----------------------------
// START: Curseforge API Files Search
// -----------------------------

pub async fn download_mods_from_curseforge(
    files: &[CurseforgeFile],
    instance_name: &str,
    instances_dir: &Path,
) -> DownloadResult<()> {
    debug!("download_mods_from_curseforge");
    // Send request with headers and body content.
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());
    header_map.insert("Accept", "application/json".parse().unwrap());

    let file_ids: Vec<u32> = files.into_iter().map(|file| file.file_id).collect();

    let url = format!("{}/mods/files", CURSEFORGE_API_URL);
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .headers(header_map)
        .body(json!({ "fileIds": file_ids }).to_string())
        .send()
        .await?;

    let response = response.json::<CurseforgeFilesResponse>().await?;
    let mut download_queue: Vec<Box<dyn Downloadable + Send + Sync>> = Vec::new();
    let mut dependencies: Vec<u32> = Vec::new();
    for files_data in response.data {
        for possible_dependency in &files_data.dependencies {
            // Possible enum values:
            // 1=EmbeddedLibrary
            // 2=OptionalDependency
            // 3=RequiredDependency
            // 4=Tool
            // 5=Incompatible
            // 6=Include
            // If we have a required dependency then add it to dependencies list.
            if possible_dependency.relation_type == 3 {
                dependencies.push(possible_dependency.mod_id);
            }
        }
        download_queue.push(Box::new(files_data));
    }
    // FIXME: Get the downloadurls from the dependencies list using
    // https://api.curseforge.com/v1/mods/{modid}
    let mods_dir = instances_dir.join(instance_name).join("mods");

    // Download all the files
    boxed_buffered_download_stream(&download_queue, &mods_dir, |bytes, downloadable| {
        if !validate_hash_sha1(&bytes, &downloadable.hash()) {
            let err = format!("Error downloading {}, invalid hash.", &downloadable.url());
            error!("{}", err);
            return Err(DownloadError::InvalidFileHashError(err));
        }
        debug!("Downloading mod: {}", downloadable.name());
        let path = downloadable.path(&mods_dir);
        let mut file = File::create(&path)?;
        file.write_all(&bytes)?;
        Ok(())
    })
    .await?;

    Ok(())
}

async fn download_dependencies_recursively(modid: u32) -> reqwest::Result<()> {
    let search_entry = download_mod_from_modid(modid).await?;

    Ok(())
}


async fn download_mod_from_modid(modid: u32) -> reqwest::Result<CurseforgeSearchEntry> {
    // TODO: Change this endpoint to `mods/{modid}/files to use pagination and get the first file with the matching game version
    let url = format!("{}/mods/{}", CURSEFORGE_API_URL, modid);
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());
    header_map.insert("Accept", "application/json".parse().unwrap());

    #[derive(Debug, Deserialize)]
    struct SingleModSearch {
        data: CurseforgeSearchEntry
    }

    let x = download_json_object_with_headers::<SingleModSearch>(&url, header_map).await?;
    Ok(x.data)
}

#[test]
fn test_download_mod_from_modid() {
    block_on(download_mod_from_modid(320926)).unwrap();
}

#[derive(Debug, Deserialize)]
struct CurseforgeFilesResponse {
    data: Vec<CurseforgeFilesData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeFilesData {
    id: u32,
    game_id: u32,
    mod_id: u32,
    is_available: bool,
    display_name: String,
    file_name: String,
    release_type: u8,
    file_status: u8,
    hashes: Vec<CurseforgeHash>,
    file_date: String,
    file_length: u32,
    download_count: u32,
    // This can be null when the mod author does not allow 3rd partys to automatically download the mod.
    download_url: Option<String>,
    game_versions: Vec<String>,
    sortable_game_versions: Vec<CurseforgeSortableGameVersion>,
    dependencies: Vec<CurseforgeDependency>,
    alternate_file_id: u32,
    is_server_pack: bool,
    file_fingerprint: u32,
    modules: Vec<CurseforgeModule>,
}

impl Downloadable for CurseforgeFilesData {
    fn name(&self) -> &str {
        &self.file_name
    }

    fn url(&self) -> String {
        match &self.download_url {
            Some(url) => url.into(),
            None => {
                let num_str = self.id.to_string();
                let parts = num_str.split_at(4);
                format!(
                    "{}/{}/{}/{}",
                    CURSEFORGE_FORGECDN_URL, parts.0, parts.1, self.file_name
                )
            }
        }
    }

    fn hash(&self) -> &str {
        &self
            .hashes
            .iter()
            .find(|hash| hash.algo == 1)
            .unwrap()
            .value
    }

    fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.file_name)
    }
}

#[derive(Debug, Deserialize)]
struct CurseforgeModule {
    name: String,
    fingerprint: u32,
}

// -----------------------------
// END: Curseforge API Files Search
// -----------------------------

// -----------------------------------------
// START: Curseforge API Mod/Modpack Search
// -----------------------------------------

#[derive(Debug, Deserialize)]
struct CurseforgeSearchResponse {
    data: Vec<CurseforgeSearchEntry>,
    pagination: CurseforgeSearchPagination,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchEntry {
    id: u32,
    game_id: u32,
    name: String,
    slug: String,
    links: CurseforgeSearchEntryLinks,
    summary: String,
    status: u32,
    download_count: u32,
    is_featured: bool,
    primary_category_id: u32,
    categories: Vec<CurseforgeSearchCategory>,
    class_id: u32,
    authors: Vec<CurseforgeSearchAuthors>,
    logo: CurseforgeSearchImage,
    screenshots: Vec<CurseforgeSearchImage>,
    main_file_id: u32,
    latest_files: Vec<CurseforgeSearchLatestFiles>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchLatestFiles {
    id: u32,
    game_id: u32,
    mod_id: u32,
    is_available: bool,
    display_name: String,
    file_name: String,
    release_type: u32,
    file_status: u32,
    hashes: Vec<CurseforgeHash>,
    file_date: String,
    file_length: u32,
    download_count: u32,
    download_url: Option<String>,
    game_versions: Vec<String>,
    sortable_game_versions: Vec<CurseforgeSortableGameVersion>,
    dependencies: Vec<CurseforgeDependency>,
    alternate_file_id: u32,
    is_server_pack: bool,
    server_pack_file_id: Option<u32>,
    file_fingerprint: u32,
    modules: Vec<CurseforgeSearchModule>,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchModule {
    name: String,
    fingerprint: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchImage {
    id: u32,
    mod_id: u32,
    title: String,
    description: String,
    thumbnail_url: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct CurseforgeSearchAuthors {
    id: u32,
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchCategory {
    id: u32,
    game_id: u32,
    name: String,
    slug: String,
    url: String,
    icon_url: String,
    date_modified: String,
    is_class: bool,
    class_id: u32,
    parent_category_id: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchEntryLinks {
    website_url: Option<String>,
    wiki_url: Option<String>,
    issues_url: Option<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseforgeSearchPagination {
    index: u32,
    page_size: u32,
    result_count: u32,
    total_count: u32,
}

async fn search_curseforge_modpacks() -> reqwest::Result<CurseforgeSearchResponse> {
    let mut header_map = HeaderMap::new();
    header_map.insert(
        "X-API-KEY",
        "$2a$10$5BgCleD8.rLQ5Ix17Xm2lOjgfoeTJV26a1BXmmpwrOemgI517.nuC"
            .parse()
            .unwrap(),
    );
    header_map.insert("Content-Type", "application/json".parse().unwrap());

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/mods/search", CURSEFORGE_API_URL))
        .headers(header_map)
        .query(&[
            ("gameId", "432"),
            ("categoryId", "0"),
            ("pageSize", "40"),
            ("index", "0"),
            ("sortField", "1"),
            ("sortOrder", "desc"),
            ("classId", "4471"),
        ])
        .send()
        .await?;
    response.json::<CurseforgeSearchResponse>().await
}

#[test]
fn test_curseforge_search() {
    let x = block_on(search_curseforge_modpacks()).unwrap();
    println!("Here: {:#?}", x);
}

// -----------------------------------------
// END: Curseforge API Mod/Modpack Search
// -----------------------------------------
